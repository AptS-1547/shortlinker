use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection, EntityTrait,
    QueryFilter,
};
use tracing::{error, info, trace, warn};

use crate::errors::{Result, ShortlinkerError};
use crate::repository::click::ClickSink;
use crate::repository::models::StorageConfig;
use crate::repository::{Repository, ShortLink};

use migration::{Migrator, MigratorTrait, entities::short_link};

#[derive(Clone)]
pub struct SeaOrmRepository {
    db: DatabaseConnection,
    backend_name: String,
}

impl SeaOrmRepository {
    pub async fn new(database_url: &str, backend_name: &str) -> Result<Self> {
        if database_url.is_empty() {
            return Err(ShortlinkerError::database_config(
                "DATABASE_URL 未设置".to_string(),
            ));
        }

        // 根据不同数据库类型配置连接选项
        let db = if backend_name == "sqlite" {
            Self::connect_sqlite(database_url).await?
        } else {
            Self::connect_generic(database_url, backend_name).await?
        };

        let repository = SeaOrmRepository {
            db,
            backend_name: backend_name.to_string(),
        };

        // 运行迁移
        repository.run_migrations().await?;

        warn!(
            "{} Repository initialized.",
            repository.backend_name.to_uppercase()
        );
        Ok(repository)
    }

    /// 连接 SQLite 数据库（带自动创建和性能优化）
    async fn connect_sqlite(database_url: &str) -> Result<DatabaseConnection> {
        use sea_orm::SqlxSqliteConnector;
        use sea_orm::sqlx::SqlitePool;
        use sea_orm::sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
        use std::str::FromStr;

        let opt = SqliteConnectOptions::from_str(database_url)
            .map_err(|e| ShortlinkerError::database_config(format!("SQLite URL 解析失败: {}", e)))?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal)
            .busy_timeout(std::time::Duration::from_secs(5))
            .pragma("cache_size", "-64000")
            .pragma("temp_store", "memory")
            .pragma("mmap_size", "536870912")
            .pragma("wal_autocheckpoint", "1000");

        // 使用 sqlx 的连接池
        let pool = SqlitePool::connect_with(opt).await.map_err(|e| {
            ShortlinkerError::database_connection(format!("无法连接到 SQLite 数据库: {}", e))
        })?;

        // 转换为 Sea-ORM 的 DatabaseConnection
        Ok(SqlxSqliteConnector::from_sqlx_sqlite_pool(pool))
    }

    /// 连接通用数据库（MySQL/PostgreSQL）
    async fn connect_generic(database_url: &str, backend_name: &str) -> Result<DatabaseConnection> {
        let mut opt = ConnectOptions::new(database_url.to_owned());
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(std::time::Duration::from_secs(8))
            .acquire_timeout(std::time::Duration::from_secs(8))
            .idle_timeout(std::time::Duration::from_secs(8))
            .max_lifetime(std::time::Duration::from_secs(8))
            .sqlx_logging(false);

        Database::connect(opt).await.map_err(|e| {
            ShortlinkerError::database_connection(format!(
                "无法连接到 {} 数据库: {}",
                backend_name.to_uppercase(),
                e
            ))
        })
    }

    async fn run_migrations(&self) -> Result<()> {
        Migrator::up(&self.db, None)
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("迁移失败: {}", e)))?;

        info!("Database migrations completed");
        Ok(())
    }

    /// 将 Sea-ORM Model 转换为 ShortLink
    fn model_to_shortlink(model: short_link::Model) -> ShortLink {
        ShortLink {
            code: model.short_code,
            target: model.target_url,
            created_at: model.created_at,
            expires_at: model.expires_at,
            password: model.password,
            click: model.click_count.max(0) as usize,
        }
    }

    /// 将 ShortLink 转换为 ActiveModel（用于插入/更新）
    fn shortlink_to_active_model(link: &ShortLink, is_new: bool) -> short_link::ActiveModel {
        use sea_orm::ActiveValue::*;

        short_link::ActiveModel {
            short_code: Set(link.code.clone()),
            target_url: Set(link.target.clone()),
            created_at: if is_new { Set(link.created_at) } else { NotSet },
            expires_at: Set(link.expires_at),
            password: Set(link.password.clone()),
            click_count: if is_new { Set(0) } else { NotSet },
        }
    }

    /// 使用 ON CONFLICT 的原子 upsert
    async fn upsert_with_on_conflict(&self, link: &ShortLink) -> Result<()> {
        use sea_orm::InsertResult;
        use sea_orm::sea_query::OnConflict;

        let active_model = Self::shortlink_to_active_model(link, true);

        let result: std::result::Result<InsertResult<short_link::ActiveModel>, sea_orm::DbErr> =
            short_link::Entity::insert(active_model)
                .on_conflict(
                    OnConflict::column(short_link::Column::ShortCode)
                        .update_columns([
                            short_link::Column::TargetUrl,
                            short_link::Column::ExpiresAt,
                            short_link::Column::Password,
                        ])
                        .to_owned(),
                )
                .exec(&self.db)
                .await;

        match result {
            Ok(_) => {
                info!("Short link upserted: {}", link.code);
                Ok(())
            }
            Err(e) => Err(ShortlinkerError::database_operation(format!(
                "Upsert 短链接失败: {}",
                e
            ))),
        }
    }

    /// 使用 try-insert-then-update 的 upsert
    async fn upsert_with_fallback(&self, link: &ShortLink) -> Result<()> {
        // 先尝试插入
        let active_model = Self::shortlink_to_active_model(link, true);
        let insert_result = active_model.clone().insert(&self.db).await;

        match insert_result {
            Ok(_) => {
                info!("Short link created: {}", link.code);
                Ok(())
            }
            Err(sea_orm::DbErr::Exec(sea_orm::RuntimeErr::SqlxError(sqlx_err))) => {
                // 检查是否是唯一约束冲突错误
                if Self::is_unique_violation(&sqlx_err) {
                    // 如果是唯一冲突，执行更新
                    let update_model = Self::shortlink_to_active_model(link, false);
                    update_model.update(&self.db).await.map_err(|e| {
                        ShortlinkerError::database_operation(format!("更新短链接失败: {}", e))
                    })?;
                    info!("Short link updated: {}", link.code);
                    Ok(())
                } else {
                    // 其他错误直接返回
                    Err(ShortlinkerError::database_operation(format!(
                        "插入短链接失败: {}",
                        sqlx_err
                    )))
                }
            }
            Err(e) => Err(ShortlinkerError::database_operation(format!(
                "插入短链接失败: {}",
                e
            ))),
        }
    }

    /// 判断是否是唯一约束冲突错误
    fn is_unique_violation(err: &sea_orm::sqlx::Error) -> bool {
        use sea_orm::sqlx::Error;

        match err {
            Error::Database(db_err) => {
                let code = db_err.code();
                // SQLite: SQLITE_CONSTRAINT (code 2067)
                // MySQL: ER_DUP_ENTRY (code 1062)
                // PostgreSQL: unique_violation (code 23505)
                code.as_ref()
                    .map(|c| {
                        c == "2067"  // SQLite
                            || c == "1062"  // MySQL
                            || c == "23505" // PostgreSQL
                    })
                    .unwrap_or(false)
            }
            _ => false,
        }
    }
}

#[async_trait]
impl Repository for SeaOrmRepository {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        let result = short_link::Entity::find_by_id(code).one(&self.db).await;

        match result {
            Ok(Some(model)) => Some(Self::model_to_shortlink(model)),
            Ok(None) => None,
            Err(e) => {
                error!("查询短链接失败: {}", e);
                None
            }
        }
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        let mut links = HashMap::new();

        match short_link::Entity::find().all(&self.db).await {
            Ok(models) => {
                for model in models {
                    let code = model.short_code.clone();
                    links.insert(code, Self::model_to_shortlink(model));
                }
            }
            Err(e) => {
                error!("加载所有短链接失败: {}", e);
            }
        }

        info!("Loaded {} short links", links.len());
        links
    }

    async fn set(&self, link: ShortLink) -> Result<()> {
        // 使用原子化的 upsert 操作，避免竞态条件
        match self.backend_name.as_str() {
            "sqlite" | "postgres" | "mysql" => {
                // These backends support ON CONFLICT or equivalent upsert syntax.
                self.upsert_with_on_conflict(&link).await
            }
            _ => {
                // Fallback for other databases, if any.
                warn!("Using fallback upsert for backend: {}", self.backend_name);
                self.upsert_with_fallback(&link).await
            }
        }
    }

    async fn remove(&self, code: &str) -> Result<()> {
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

    async fn reload(&self) -> Result<()> {
        info!(
            "Reloading links from {} storage",
            self.backend_name.to_uppercase()
        );
        Ok(())
    }

    async fn get_backend_config(&self) -> StorageConfig {
        StorageConfig {
            storage_type: self.backend_name.clone(),
            support_click: true,
        }
    }

    fn as_click_sink(&self) -> Option<Arc<dyn ClickSink>>
    where
        Self: Clone + Sized,
    {
        Some(Arc::new(self.clone()) as Arc<dyn ClickSink>)
    }
}

#[async_trait]
impl ClickSink for SeaOrmRepository {
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
