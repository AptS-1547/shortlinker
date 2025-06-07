use std::collections::HashMap;
use std::env;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use postgres::{Client, NoTls};
use r2d2::{Pool, PooledConnection};
use r2d2_postgres::{postgres::Config, PostgresConnectionManager};
use tracing::{error, info, warn};

use crate::errors::{Result, ShortlinkerError};
use crate::storages::{ShortLink, Storage, CachePreference};

declare_storage_plugin!("postgres", PostgresStorage);

pub struct PostgresStorage {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PostgresStorage {
    pub async fn new_async() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ShortlinkerError::database_config("DATABASE_URL not set"))?;

        let config: Config = database_url.parse().map_err(|e| {
            ShortlinkerError::database_config(format!("Failed to parse DATABASE_URL: {}", e))
        })?;

        let manager = PostgresConnectionManager::new(config, NoTls);
        let pool = Pool::new(manager)
            .map_err(|e| ShortlinkerError::database_connection(format!("{}", e)))?;

        let storage = PostgresStorage { pool };
        storage.init_db()?;

        warn!("PostgresStorage initialized using {}", database_url);
        Ok(storage)
    }

    fn get_connection(&self) -> Result<PooledConnection<PostgresConnectionManager<NoTls>>> {
        self.pool.get().map_err(|e| {
            ShortlinkerError::database_connection(format!("Failed to get Postgres connection: {}", e))
        })
    }

    fn init_db(&self) -> Result<()> {
        let mut conn = self.get_connection()?;
        conn.batch_execute(
            "CREATE TABLE IF NOT EXISTS short_links (
                short_code TEXT PRIMARY KEY,
                target_url TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL,
                expires_at TIMESTAMPTZ,
                click_count BIGINT DEFAULT 0
            );"
        ).map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to create table: {}", e))
        })?;
        Ok(())
    }

    fn shortlink_from_row(
        short_code: String,
        target_url: String,
        created_at: DateTime<Utc>,
        expires_at: Option<DateTime<Utc>>,
    ) -> ShortLink {
        ShortLink {
            code: short_code,
            target: target_url,
            created_at,
            expires_at,
        }
    }
}

#[async_trait]
impl Storage for PostgresStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        let code = code.to_string();
        let result = tokio::task::spawn_blocking(move || {
            let mut conn = self.get_connection()?;
            conn.query_opt(
                "SELECT short_code, target_url, created_at, expires_at FROM short_links WHERE short_code = $1",
                &[&code],
            ).map_err(|e| {
                ShortlinkerError::database_operation(format!("Query failed: {}", e))
            })
        }).await.ok().flatten();

        result.ok().flatten().map(|row| {
            let short_code: String = row.get(0);
            let target_url: String = row.get(1);
            let created_at: DateTime<Utc> = row.get(2);
            let expires_at: Option<DateTime<Utc>> = row.get(3);
            Self::shortlink_from_row(short_code, target_url, created_at, expires_at)
        })
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        let mut result = HashMap::new();
        if let Ok(mut conn) = self.get_connection() {
            if let Ok(rows) = conn.query("SELECT short_code, target_url, created_at, expires_at FROM short_links", &[]) {
                for row in rows {
                    let short_code: String = row.get(0);
                    let link = Self::shortlink_from_row(
                        short_code.clone(),
                        row.get(1),
                        row.get(2),
                        row.get(3),
                    );
                    result.insert(short_code, link);
                }
            }
        }
        result
    }

    async fn set(&self, link: ShortLink) -> Result<()> {
        let code = link.code.clone();
        let target = link.target.clone();
        let created_at = link.created_at;
        let expires_at = link.expires_at;

        tokio::task::spawn_blocking(move || {
            let mut conn = self.get_connection()?;
            conn.execute(
                "INSERT INTO short_links (short_code, target_url, created_at, expires_at)
                 VALUES ($1, $2, $3, $4)
                 ON CONFLICT(short_code) DO UPDATE SET target_url = $2, expires_at = $4",
                &[&code, &target, &created_at, &expires_at],
            ).map_err(|e| {
                ShortlinkerError::database_operation(format!("Insert/Update failed: {}", e))
            })?;
            Ok(())
        }).await.map_err(|e| {
            ShortlinkerError::internal(format!("Blocking task error: {:?}", e))
        })?
    }

    async fn remove(&self, code: &str) -> Result<()> {
        let code = code.to_string();
        tokio::task::spawn_blocking(move || {
            let mut conn = self.get_connection()?;
            conn.execute(
                "DELETE FROM short_links WHERE short_code = $1",
                &[&code],
            ).map_err(|e| {
                ShortlinkerError::database_operation(format!("Delete failed: {}", e))
            })?;
            Ok(())
        }).await.map_err(|e| {
            ShortlinkerError::internal(format!("Blocking task error: {:?}", e))
        })?
    }

    async fn reload(&self) -> Result<()> {
        Ok(())
    }

    async fn get_backend_name(&self) -> String {
        "postgres".into()
    }

    async fn increment_click(&self, code: &str) -> Result<()> {
        let code = code.to_string();
        tokio::task::spawn_blocking(move || {
            let mut conn = self.get_connection()?;
            conn.execute(
                "UPDATE short_links SET click_count = click_count + 1 WHERE short_code = $1",
                &[&code],
            ).map_err(|e| {
                ShortlinkerError::database_operation(format!("Click increment failed: {}", e))
            })?;
            Ok(())
        }).await.map_err(|e| {
            ShortlinkerError::internal(format!("Blocking task error: {:?}", e))
        })?
    }

    fn preferred_cache(&self) -> crate::storages::CachePreference {
        CachePreference {
            l1: "bloom".into(),
            l2: "moka".into(),
        }
    }
}
