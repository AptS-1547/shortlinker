mod connection;
mod converters;
mod operations;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::{error, info, trace, warn};

use crate::analytics::ClickSink;
use crate::errors::{Result, ShortlinkerError};
use crate::storage::ShortLink;
use crate::storage::models::StorageConfig;

use migration::entities::short_link;

pub use connection::{connect_generic, connect_sqlite, run_migrations};
pub use converters::{model_to_shortlink, shortlink_to_active_model};
pub use operations::upsert;

/// 从数据库 URL 推断数据库类型
pub fn infer_backend_from_url(database_url: &str) -> Result<String> {
    if database_url.starts_with("sqlite://")
        || database_url.ends_with(".db")
        || database_url.ends_with(".sqlite")
        || database_url == ":memory:"
    {
        Ok("sqlite".to_string())
    } else if database_url.starts_with("mysql://") || database_url.starts_with("mariadb://") {
        Ok("mysql".to_string())
    } else if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        Ok("postgres".to_string())
    } else {
        Err(ShortlinkerError::database_config(format!(
            "无法从 URL 推断数据库类型: {}. 支持的 URL 格式: sqlite://, mysql://, mariadb://, postgres://",
            database_url
        )))
    }
}

/// 规范化 backend 名称
pub fn normalize_backend_name(backend: &str) -> String {
    match backend {
        "mariadb" => "mysql".to_string(),
        other => other.to_string(),
    }
}

#[derive(Clone)]
pub struct SeaOrmStorage {
    db: DatabaseConnection,
    backend_name: String,
}

impl SeaOrmStorage {
    pub async fn new(database_url: &str, backend_name: &str) -> Result<Self> {
        if database_url.is_empty() {
            return Err(ShortlinkerError::database_config(
                "DATABASE_URL 未设置".to_string(),
            ));
        }

        // 根据不同数据库类型配置连接选项
        let db = if backend_name == "sqlite" {
            connect_sqlite(database_url).await?
        } else {
            connect_generic(database_url, backend_name).await?
        };

        let storage = SeaOrmStorage {
            db,
            backend_name: backend_name.to_string(),
        };

        // 运行迁移
        run_migrations(&storage.db).await?;

        warn!(
            "{} Storage initialized.",
            storage.backend_name.to_uppercase()
        );
        Ok(storage)
    }

    pub async fn get(&self, code: &str) -> Option<ShortLink> {
        let result = short_link::Entity::find_by_id(code).one(&self.db).await;

        match result {
            Ok(Some(model)) => Some(model_to_shortlink(model)),
            Ok(None) => None,
            Err(e) => {
                error!("查询短链接失败: {}", e);
                None
            }
        }
    }

    pub async fn load_all(&self) -> HashMap<String, ShortLink> {
        let mut links = HashMap::new();

        match short_link::Entity::find().all(&self.db).await {
            Ok(models) => {
                for model in models {
                    let code = model.short_code.clone();
                    links.insert(code, model_to_shortlink(model));
                }
            }
            Err(e) => {
                error!("加载所有短链接失败: {}", e);
            }
        }

        info!("Loaded {} short links", links.len());
        links
    }

    pub async fn set(&self, link: ShortLink) -> Result<()> {
        upsert(&self.db, &self.backend_name, &link).await
    }

    pub async fn remove(&self, code: &str) -> Result<()> {
        let result = short_link::Entity::delete_by_id(code)
            .exec(&self.db)
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("删除短链接失败: {}", e)))?;

        if result.rows_affected == 0 {
            return Err(ShortlinkerError::not_found(format!(
                "短链接不存在: {}",
                code
            )));
        }

        info!("Short link deleted: {}", code);
        Ok(())
    }

    pub async fn reload(&self) -> Result<()> {
        info!(
            "Reloading links from {} storage",
            self.backend_name.to_uppercase()
        );
        Ok(())
    }

    pub async fn get_backend_config(&self) -> StorageConfig {
        StorageConfig {
            storage_type: self.backend_name.clone(),
            support_click: true,
        }
    }

    pub fn as_click_sink(&self) -> Option<Arc<dyn ClickSink>> {
        Some(Arc::new(self.clone()) as Arc<dyn ClickSink>)
    }
}

#[async_trait]
impl ClickSink for SeaOrmStorage {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
        use sea_orm::{ExprTrait, TransactionTrait, sea_query::Expr};

        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| anyhow::anyhow!("开始事务失败: {}", e))?;

        for (code, count) in updates {
            // 使用原生 SQL 进行原子增量更新
            let update_result = short_link::Entity::update_many()
                .col_expr(
                    short_link::Column::ClickCount,
                    Expr::col(short_link::Column::ClickCount).add(count as i64),
                )
                .filter(short_link::Column::ShortCode.eq(&code))
                .exec(&txn)
                .await;

            if let Err(e) = update_result {
                error!("点击计数更新失败 {}: {}", code, e);
            }
        }

        txn.commit()
            .await
            .map_err(|e| anyhow::anyhow!("提交事务失败: {}", e))?;

        trace!(
            "点击计数已刷新到 {} 数据库",
            self.backend_name.to_uppercase()
        );
        Ok(())
    }
}
