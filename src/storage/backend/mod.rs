//! SeaORM storage backend
//!
//! This module provides database storage using SeaORM,
//! supporting SQLite, MySQL/MariaDB, and PostgreSQL.

mod click_sink;
mod connection;
mod converters;
mod mutations;
mod operations;
mod query;
pub mod retry;

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use moka::sync::Cache;
use sea_orm::DatabaseConnection;
use tracing::warn;

use crate::analytics::ClickSink;
use crate::errors::{Result, ShortlinkerError};
use crate::storage::models::StorageConfig;

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

/// 链接过滤条件
#[derive(Default, Clone, Debug)]
pub struct LinkFilter {
    /// 模糊搜索 code 或 target
    pub search: Option<String>,
    /// 创建时间 >= created_after
    pub created_after: Option<DateTime<Utc>>,
    /// 创建时间 <= created_before
    pub created_before: Option<DateTime<Utc>>,
    /// 只返回已过期的链接
    pub only_expired: bool,
    /// 只返回未过期的链接
    pub only_active: bool,
}

/// SeaORM-based storage backend
#[derive(Clone)]
pub struct SeaOrmStorage {
    db: DatabaseConnection,
    backend_name: String,
    /// 分页 COUNT 缓存（TTL 30秒）
    count_cache: Cache<String, u64>,
    /// 重试配置
    retry_config: retry::RetryConfig,
}

impl SeaOrmStorage {
    pub async fn new(database_url: &str, backend_name: &str) -> Result<Self> {
        if database_url.is_empty() {
            return Err(ShortlinkerError::database_config(
                "DATABASE_URL 未设置".to_string(),
            ));
        }

        // 读取重试配置
        let config = crate::config::get_config();
        let retry_config = retry::RetryConfig {
            max_retries: config.database.retry_count,
            base_delay_ms: config.database.retry_base_delay_ms,
            max_delay_ms: config.database.retry_max_delay_ms,
        };

        // 根据不同数据库类型配置连接选项
        let db = if backend_name == "sqlite" {
            connect_sqlite(database_url).await?
        } else {
            connect_generic(database_url, backend_name).await?
        };

        let storage = SeaOrmStorage {
            db,
            backend_name: backend_name.to_string(),
            count_cache: Cache::builder()
                .time_to_live(Duration::from_secs(30))
                .max_capacity(100)
                .build(),
            retry_config,
        };

        // 运行迁移
        run_migrations(&storage.db).await?;

        warn!(
            "{} Storage initialized.",
            storage.backend_name.to_uppercase()
        );
        Ok(storage)
    }

    pub async fn reload(&self) -> Result<()> {
        tracing::info!(
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

    /// 获取数据库连接（用于配置系统等需要直接访问数据库的场景）
    pub fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    /// 清除分页 COUNT 缓存（数据变更时调用）
    pub fn invalidate_count_cache(&self) {
        self.count_cache.invalidate_all();
    }
}
