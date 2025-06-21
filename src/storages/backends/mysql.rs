use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{MySqlPool, Row};
use tracing::{debug, error, info, warn};

use crate::errors::{Result, ShortlinkerError};
use crate::storages::click::ClickSink;
use crate::storages::models::StorageConfig;
use crate::storages::{CachePreference, ShortLink, Storage};

declare_storage_plugin!("mysql", MySqlStorage);

#[ctor::ctor]
fn register_mariadb_alias() {
    use crate::storages::{register::register_storage_plugin, Storage};
    use std::{future::Future, pin::Pin, sync::Arc};

    register_storage_plugin(
        "mariadb",
        Arc::new(|| {
            Box::pin(async {
                let storage = MySqlStorage::new_async().await?;
                Ok(Box::new(storage) as Box<dyn Storage>)
            })
                as Pin<Box<dyn Future<Output = crate::errors::Result<Box<dyn Storage>>> + Send>>
        }),
    );
}

#[derive(Clone)]
pub struct MySqlStorage {
    pool: MySqlPool,
    db_type: String, // "mysql" 或 "mariadb"
}

impl MySqlStorage {
    pub async fn new_async() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ShortlinkerError::database_config("DATABASE_URL not set".to_string()))?;

        // 从 URL 检测数据库类型
        let db_type = if database_url.contains("mariadb")
            || env::var("STORAGE_BACKEND").unwrap_or_default() == "mariadb"
        {
            "mariadb".to_string()
        } else {
            "mysql".to_string()
        };

        // 创建连接池
        let pool = MySqlPool::connect(&database_url).await.map_err(|e| {
            ShortlinkerError::database_connection(format!(
                "无法连接到{}数据库: {}",
                db_type.to_uppercase(),
                e
            ))
        })?;

        let storage = MySqlStorage {
            pool,
            db_type: db_type.clone(),
        };

        // 初始化数据库表
        storage.init_db().await?;

        warn!("{} Storage initialized.", db_type.to_uppercase());
        Ok(storage)
    }

    async fn init_db(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS short_links (
                short_code VARCHAR(255) PRIMARY KEY,
                target_url TEXT NOT NULL,
                created_at TIMESTAMP(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
                expires_at TIMESTAMP(6) NULL,
                password VARCHAR(255) NULL,
                click_count BIGINT UNSIGNED DEFAULT 0,
                INDEX idx_expires_at (expires_at),
                INDEX idx_created_at (created_at)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ShortlinkerError::database_operation(format!("创建表失败: {}", e)))?;

        Ok(())
    }

    fn shortlink_from_row(
        short_code: String,
        target_url: String,
        created_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
        password: Option<String>,
        click_count: u64,
    ) -> ShortLink {
        ShortLink {
            code: short_code,
            target: target_url,
            created_at,
            expires_at,
            password,
            click: click_count as usize, // 默认点击量为0
        }
    }
}

#[async_trait]
impl Storage for MySqlStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        let result = sqlx::query(
            "SELECT short_code, target_url, created_at, expires_at FROM short_links WHERE short_code = ?"
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await;

        match result {
            Ok(Some(row)) => {
                let short_code: String = row.get("short_code");
                let target_url: String = row.get("target_url");
                let created_at: DateTime<Utc> = row.get("created_at");
                let expires_at: Option<DateTime<Utc>> = row.get("expires_at");
                let password: Option<String> = row.get("password");
                let click_count: u64 = row.get("click_count");

                Some(Self::shortlink_from_row(
                    short_code,
                    target_url,
                    created_at,
                    expires_at,
                    password,
                    click_count,
                ))
            }
            Ok(None) => None,
            Err(e) => {
                error!("Query short link failed: {}", e);
                None
            }
        }
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        let mut result = HashMap::new();

        match sqlx::query("SELECT short_code, target_url, created_at, expires_at FROM short_links")
            .fetch_all(&self.pool)
            .await
        {
            Ok(rows) => {
                for row in rows {
                    let short_code: String = row.get("short_code");
                    let target_url: String = row.get("target_url");
                    let created_at: DateTime<Utc> = row.get("created_at");
                    let expires_at: Option<DateTime<Utc>> = row.get("expires_at");
                    let password: Option<String> = row.get("password");
                    let click_count: u64 = row.get("click_count");

                    let link = Self::shortlink_from_row(
                        short_code.clone(),
                        target_url,
                        created_at,
                        expires_at,
                        password,
                        click_count,
                    );
                    result.insert(short_code, link);
                }
            }
            Err(e) => {
                error!("Failed to load all short links: {}", e);
            }
        }

        result
    }

    async fn set(&self, link: ShortLink) -> Result<()> {
        sqlx::query(
            "INSERT INTO short_links (short_code, target_url, created_at, expires_at)
             VALUES (?, ?, ?, ?)
             ON DUPLICATE KEY UPDATE target_url = VALUES(target_url), expires_at = VALUES(expires_at)"
        )
        .bind(&link.code)
        .bind(&link.target)
        .bind(link.created_at)
        .bind(link.expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            ShortlinkerError::database_operation(format!("插入/更新短链接失败: {}", e))
        })?;

        info!("Short link upserted: {}", link.code);
        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM short_links WHERE short_code = ?")
            .bind(code)
            .execute(&self.pool)
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("删除短链接失败: {}", e)))?;

        if result.rows_affected() == 0 {
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
            self.db_type.to_uppercase()
        );
        Ok(())
    }

    async fn get_backend_config(&self) -> StorageConfig {
        StorageConfig {
            storage_type: self.db_type.clone(),
            support_click: true,
        }
    }

    fn as_click_sink(&self) -> Option<Arc<dyn ClickSink>>
    where
        Self: Clone + Sized,
    {
        Some(Arc::new(self.clone()) as Arc<dyn ClickSink>)
    }

    fn preferred_cache(&self) -> CachePreference {
        CachePreference {
            l1: "bloom".into(),
            l2: "moka".into(),
        }
    }
}

#[async_trait]
impl ClickSink for MySqlStorage {
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
            .bind(count as u64)
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

        debug!("click counts flushed to {}.", self.db_type.to_uppercase());
        Ok(())
    }
}
