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
        match short_link::Entity::find().all(&self.db).await {
            Ok(models) => {
                let count = models.len();
                // 使用 into_iter 避免不必要的克隆
                let links: HashMap<String, ShortLink> = models
                    .into_iter()
                    .map(|model| {
                        let link = model_to_shortlink(model);
                        (link.code.clone(), link)
                    })
                    .collect();
                info!("Loaded {} short links", count);
                links
            }
            Err(e) => {
                error!("加载所有短链接失败: {}", e);
                HashMap::new()
            }
        }
    }

    /// 分页加载链接
    pub async fn load_paginated(&self, page: u64, page_size: u64) -> (Vec<ShortLink>, u64) {
        use sea_orm::{PaginatorTrait, QueryOrder};

        let paginator = short_link::Entity::find()
            .order_by_desc(short_link::Column::CreatedAt)
            .paginate(&self.db, page_size);

        let total = match paginator.num_items().await {
            Ok(count) => count,
            Err(e) => {
                error!("获取总数失败: {}", e);
                return (Vec::new(), 0);
            }
        };

        let models = match paginator.fetch_page(page.saturating_sub(1)).await {
            Ok(models) => models,
            Err(e) => {
                error!("分页查询失败: {}", e);
                return (Vec::new(), total);
            }
        };

        let links: Vec<ShortLink> = models.into_iter().map(model_to_shortlink).collect();

        (links, total)
    }

    /// 批量获取链接
    pub async fn batch_get(&self, codes: &[&str]) -> HashMap<String, ShortLink> {
        if codes.is_empty() {
            return HashMap::new();
        }

        let result = short_link::Entity::find()
            .filter(short_link::Column::ShortCode.is_in(codes.iter().map(|s| s.to_string())))
            .all(&self.db)
            .await;

        match result {
            Ok(models) => models
                .into_iter()
                .map(|m| {
                    let link = model_to_shortlink(m);
                    (link.code.clone(), link)
                })
                .collect(),
            Err(e) => {
                error!("批量查询失败: {}", e);
                HashMap::new()
            }
        }
    }

    /// 批量设置链接（使用事务）
    pub async fn batch_set(&self, links: Vec<ShortLink>) -> Result<()> {
        use sea_orm::{EntityTrait, TransactionTrait, sea_query::OnConflict};

        if links.is_empty() {
            return Ok(());
        }

        let txn = self.db.begin().await.map_err(|e| {
            ShortlinkerError::database_operation(format!("开始事务失败: {}", e))
        })?;

        // 构建批量 ActiveModel
        let active_models: Vec<short_link::ActiveModel> = links
            .iter()
            .map(|link| shortlink_to_active_model(link, true))
            .collect();

        // 使用 insert_many with on_conflict
        short_link::Entity::insert_many(active_models)
            .on_conflict(
                OnConflict::column(short_link::Column::ShortCode)
                    .update_columns([
                        short_link::Column::TargetUrl,
                        short_link::Column::ExpiresAt,
                        short_link::Column::Password,
                    ])
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("批量插入失败: {}", e))
            })?;

        txn.commit().await.map_err(|e| {
            ShortlinkerError::database_operation(format!("提交事务失败: {}", e))
        })?;

        info!("批量插入 {} 条链接成功", links.len());
        Ok(())
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

        if updates.is_empty() {
            return Ok(());
        }

        let total_count = updates.len();
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| anyhow::anyhow!("开始事务失败: {}", e))?;

        let mut failed_updates: Vec<(String, String)> = Vec::new();

        for (code, count) in &updates {
            // 使用原生 SQL 进行原子增量更新
            let update_result = short_link::Entity::update_many()
                .col_expr(
                    short_link::Column::ClickCount,
                    Expr::col(short_link::Column::ClickCount).add(*count as i64),
                )
                .filter(short_link::Column::ShortCode.eq(code))
                .exec(&txn)
                .await;

            if let Err(e) = update_result {
                failed_updates.push((code.clone(), e.to_string()));
            }
        }

        // 即使有部分失败，也提交成功的更新
        txn.commit()
            .await
            .map_err(|e| anyhow::anyhow!("提交事务失败: {}", e))?;

        let success_count = total_count - failed_updates.len();

        // 报告失败情况
        if !failed_updates.is_empty() {
            error!(
                "部分点击计数更新失败 ({}/{}): {:?}",
                failed_updates.len(),
                total_count,
                failed_updates
            );
        }

        trace!(
            "点击计数已刷新到 {} 数据库 ({} 成功, {} 失败)",
            self.backend_name.to_uppercase(),
            success_count,
            failed_updates.len()
        );
        Ok(())
    }
}
