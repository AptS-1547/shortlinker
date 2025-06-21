use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use tracing::{debug, error, info, warn};

use crate::errors::{Result, ShortlinkerError};
use crate::storages::click::ClickSink;
use crate::storages::models::StorageConfig;
use crate::storages::{CachePreference, ShortLink, Storage};

declare_storage_plugin!("postgres", PostgresStorage);

#[derive(Clone)]
pub struct PostgresStorage {
    pool: PgPool,
}

impl PostgresStorage {
    pub async fn new_async() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ShortlinkerError::database_config("DATABASE_URL not set".to_string()))?;

        // 创建连接池
        let pool = PgPool::connect(&database_url).await.map_err(|e| {
            ShortlinkerError::database_connection(format!("无法连接到PostgreSQL数据库: {}", e))
        })?;

        let storage = PostgresStorage { pool };

        // 初始化数据库表
        storage.init_db().await?;

        warn!("PostgresStorage initialized. Ensure your DATABASE_URL is set correctly.");
        Ok(storage)
    }

    async fn init_db(&self) -> Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS short_links (
                short_code TEXT PRIMARY KEY,
                target_url TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                expires_at TIMESTAMPTZ,
                password TEXT,
                click_count BIGINT DEFAULT 0 CHECK (click_count >= 0)
            )",
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
            click: click_count as usize, // 将 u64 转换为 usize
        }
    }
}

#[async_trait]
impl Storage for PostgresStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        let result = sqlx::query(
            "SELECT short_code, target_url, created_at, expires_at FROM short_links WHERE short_code = $1"
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
                let click_count: i64 = row.get("click_count");

                Some(Self::shortlink_from_row(
                    short_code,
                    target_url,
                    created_at,
                    expires_at,
                    password,
                    click_count as u64, // 将 i64 转换为 u64
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
                    let click_count: i64 = row.get("click_count");

                    let link = Self::shortlink_from_row(
                        short_code.clone(),
                        target_url,
                        created_at,
                        expires_at,
                        password,
                        click_count as u64, // 将 i64 转换为 u64
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
             VALUES ($1, $2, $3, $4)
             ON CONFLICT(short_code) DO UPDATE SET target_url = $2, expires_at = $4",
        )
        .bind(&link.code)
        .bind(&link.target)
        .bind(link.created_at)
        .bind(link.expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| ShortlinkerError::database_operation(format!("插入/更新短链接失败: {}", e)))?;

        info!("Short link upserted: {}", link.code);
        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM short_links WHERE short_code = $1")
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
        info!("Reloading links from PostgreSQL storage");
        Ok(())
    }

    async fn get_backend_config(&self) -> StorageConfig {
        StorageConfig {
            storage_type: "postgres".into(),
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
impl ClickSink for PostgresStorage {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start transaction: {}", e))?;

        for (code, count) in updates {
            if let Err(e) = sqlx::query(
                "UPDATE short_links SET click_count = click_count + $1 WHERE short_code = $2",
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

        debug!("click counts flushed to PostgreSQL DB.");
        Ok(())
    }
}
