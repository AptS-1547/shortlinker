use sqlx::{sqlite, sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use super::{ShortLink, Storage};
use crate::errors::{Result, ShortlinkerError};
use crate::storages::click::ClickSink;
use crate::storages::models::StorageConfig;
use async_trait::async_trait;

// 注册 SQLite 存储插件
// 该函数在应用启动时调用，注册 SQLite 存储插件到存储插件注册表
declare_storage_plugin!("sqlite", SqliteStorage);

#[derive(Clone)]
pub struct SqliteStorage {
    pool: SqlitePool,
}

impl SqliteStorage {
    pub async fn new_async() -> Result<Self> {
        let config = crate::system::app_config::get_config();
        let db_path = &config.database.database_url;

        // 创建连接池
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::new()
                .filename(db_path)
                .create_if_missing(true)
                .journal_mode(sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlite::SqliteSynchronous::Normal)
                .busy_timeout(std::time::Duration::from_secs(5))
                .pragma("cache_size", "-64000")
                .pragma("temp_store", "memory")
                .pragma("mmap_size", "536870912")
                .pragma("wal_autocheckpoint", "1000"),
        )
        .await
        .map_err(|e| ShortlinkerError::database_connection(format!("无法连接到数据库: {}", e)))?;

        let storage = SqliteStorage { pool };

        // Initialize database tables
        storage.init_db().await?;
        warn!("SqliteStorage initialized, database path: {}", db_path);

        Ok(storage)
    }

    async fn init_db(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS short_links (
                short_code TEXT PRIMARY KEY,
                target_url TEXT NOT NULL,
                created_at TEXT NOT NULL,
                expires_at TEXT,
                password TEXT,
                click_count INTEGER DEFAULT 0 CHECK (click_count >= 0)
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ShortlinkerError::database_operation(format!("创建表失败: {}", e)))?;

        // 为过期时间添加索引，用于快速查找过期链接
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_expires_at ON short_links(expires_at)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("创建过期时间索引失败: {}", e))
            })?;

        // 为创建时间添加索引，用于按时间排序查询
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_created_at ON short_links(created_at)")
            .execute(&self.pool)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("创建时间索引失败: {}", e))
            })?;

        Ok(())
    }

    fn shortlink_from_row(
        short_code: String,
        target_url: String,
        created_at: String,
        expires_at: Option<String>,
        password: Option<String>,
        click_count: usize,
    ) -> Result<ShortLink> {
        let created_at = chrono::DateTime::parse_from_rfc3339(&created_at)
            .map_err(|e| ShortlinkerError::date_parse(format!("创建时间解析失败: {}", e)))?
            .with_timezone(&chrono::Utc);

        let expires_at = expires_at.and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .ok()
        });

        Ok(ShortLink {
            code: short_code,
            target: target_url,
            created_at,
            expires_at,
            password,
            click: click_count,
        })
    }
}

#[async_trait]
impl Storage for SqliteStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        let result = sqlx::query(
            "SELECT short_code, target_url, created_at, expires_at, password, click_count
             FROM short_links 
             WHERE short_code = ?",
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let short_code: String = row.get("short_code");
                let target_url: String = row.get("target_url");
                let created_at: String = row.get("created_at");
                let expires_at: Option<String> = row.get("expires_at");
                let password: Option<String> = row.get("password");
                let click_count: i64 = row.get("click_count");

                match Self::shortlink_from_row(
                    short_code,
                    target_url,
                    created_at,
                    expires_at,
                    password,
                    click_count.max(0) as usize,
                ) {
                    Ok(link) => Some(link),
                    Err(e) => {
                        error!("Failed to parse short link data: {}", e);
                        None
                    }
                }
            }
            Ok(None) => None,
            Err(e) => {
                error!("Query short link failed: {}", e);
                None
            }
        }
    }

    // NOTE: load_all 不更新 bloom filter，因为主要用于非 HTTP 路径（例如 CLI export）
    // 如果未来在 get() 中依赖它，需要手动触发 reload() 逻辑
    async fn load_all(&self) -> HashMap<String, ShortLink> {
        let mut links = HashMap::new();

        let result = sqlx::query(
            "SELECT short_code, target_url, created_at, expires_at, password, click_count
             FROM short_links",
        )
        .fetch_all(&self.pool)
        .await;

        match result {
            Ok(rows) => {
                for row in rows {
                    let short_code: String = row.get("short_code");
                    let target_url: String = row.get("target_url");
                    let created_at: String = row.get("created_at");
                    let expires_at: Option<String> = row.get("expires_at");
                    let password: Option<String> = row.get("password");
                    let click_count: i64 = row.get("click_count");

                    match Self::shortlink_from_row(
                        short_code.clone(),
                        target_url,
                        created_at,
                        expires_at,
                        password,
                        click_count.max(0) as usize,
                    ) {
                        Ok(link) => {
                            links.insert(short_code, link);
                        }
                        Err(e) => {
                            error!("Failed to parse short link data: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to query all short links: {}", e);
            }
        }

        info!("Loaded {} short links", links.len());
        links
    }

    async fn set(&self, link: ShortLink) -> Result<()> {
        let created_at = link.created_at.to_rfc3339();
        let expires_at = link.expires_at.map(|dt| dt.to_rfc3339());

        // 开始事务
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                ShortlinkerError::database_operation(format!("开始事务失败: {}", e))
            })?;

        // 先检查记录是否存在
        let exists = sqlx::query("SELECT 1 FROM short_links WHERE short_code = ?")
            .bind(&link.code)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("检查记录存在性失败: {}", e))
            })?
            .is_some();

        if exists {
            // 更新现有记录
            sqlx::query(
                "UPDATE short_links SET target_url = ?, expires_at = ?, password = ? WHERE short_code = ?",
            )
            .bind(&link.target)
            .bind(&expires_at)
            .bind(&link.password)
            .bind(&link.code)
            .execute(&mut *tx)
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("更新短链接失败: {}", e)))?;

            info!("Short link updated: {}", link.code);
        } else {
            // 插入新记录
            sqlx::query(
                "INSERT INTO short_links (short_code, target_url, created_at, expires_at, password, click_count) VALUES (?, ?, ?, ?, ?, 0)"
            )
            .bind(&link.code)
            .bind(&link.target)
            .bind(&created_at)
            .bind(&expires_at)
            .bind(&link.password)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("插入短链接失败: {}", e))
            })?;

            info!("Short link created: {}", link.code);
        }

        // 提交事务
        tx.commit()
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("提交事务失败: {}", e)))?;

        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<()> {
        // 开始事务
        let mut tx =
            self.pool.begin().await.map_err(|e| {
                ShortlinkerError::database_operation(format!("开始事务失败: {}", e))
            })?;

        let result = sqlx::query("DELETE FROM short_links WHERE short_code = ?")
            .bind(code)
            .execute(&mut *tx)
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("删除短链接失败: {}", e)))?;

        if result.rows_affected() == 0 {
            // 事务会自动回滚
            return Err(ShortlinkerError::not_found(format!(
                "短链接不存在: {}",
                code
            )));
        }

        // 提交事务
        tx.commit()
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("提交事务失败: {}", e)))?;

        info!("Short link deleted: {}", code);
        Ok(())
    }

    async fn reload(&self) -> Result<()> {
        info!("Reloading links from SQLite storage");
        Ok(())
    }

    async fn get_backend_config(&self) -> StorageConfig {
        StorageConfig {
            storage_type: "sqlite".into(),
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
impl ClickSink for SqliteStorage {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start transaction: {}", e))?;

        for (code, count) in updates {
            if let Err(e) = sqlx::query(
                "UPDATE short_links SET click_count = click_count + ? WHERE short_code = ?",
            )
            .bind(count as i64)
            .bind(&code)
            .execute(&mut *tx)
            .await
            {
                error!("click flush: failed to write for {}: {}", code, e);
            }
        }

        tx.commit()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to commit: {}", e))?;

        debug!("click counts flushed to DB.");
        Ok(())
    }
}
