use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use tracing::{info, warn};

use crate::errors::{Result, ShortlinkerError};
use crate::storage::{ConfigHistoryEntry, ConfigItem, ConfigStore, ConfigUpdateResult};

use super::update_config_by_key;

/// 全局运行时配置实例
static RUNTIME_CONFIG: OnceLock<RuntimeConfig> = OnceLock::new();

/// 运行时配置管理器
///
/// 提供从数据库加载配置并缓存到内存的功能，
/// 支持热更新和实时重载。
pub struct RuntimeConfig {
    cache: Arc<RwLock<HashMap<String, ConfigItem>>>,
    store: Arc<ConfigStore>,
}

impl RuntimeConfig {
    /// 创建新的运行时配置实例
    pub fn new(db: DatabaseConnection) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            store: Arc::new(ConfigStore::new(db)),
        }
    }

    /// 从数据库加载所有配置到缓存，并同步到 AppConfig
    pub async fn load(&self) -> Result<()> {
        let configs = self.store.get_all().await?;
        let count = configs.len();

        // 更新内部缓存
        {
            let mut cache = self.cache.write().map_err(|_| {
                ShortlinkerError::database_operation(
                    "Cannot acquire runtime config cache write lock".to_string(),
                )
            })?;
            *cache = configs.clone();
        }

        // 同步到 AppConfig
        for (key, item) in &configs {
            update_config_by_key(key, &item.value);
        }

        info!("Loaded {} runtime configuration items from database", count);
        Ok(())
    }

    /// 重新从数据库加载配置
    pub async fn reload(&self) -> Result<()> {
        self.load().await
    }

    /// 获取配置值
    pub fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().ok()?;
        cache.get(key).map(|item| (*item.value).clone())
    }

    /// 获取配置的完整信息
    pub fn get_full(&self, key: &str) -> Option<ConfigItem> {
        let cache = self.cache.read().ok()?;
        cache.get(key).cloned()
    }

    /// 获取所有配置
    pub fn get_all(&self) -> HashMap<String, ConfigItem> {
        self.cache
            .read()
            .map(|cache| cache.clone())
            .unwrap_or_default()
    }

    /// 获取 bool 类型配置
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        let value = self.get(key)?;
        let v_lower = value.to_lowercase();
        Some(v_lower == "true" || v_lower == "1" || v_lower == "yes")
    }

    /// 获取 i64 类型配置
    pub fn get_int(&self, key: &str) -> Option<i64> {
        let value = self.get(key)?;
        value.parse().ok()
    }

    /// 获取 usize 类型配置
    pub fn get_usize(&self, key: &str) -> Option<usize> {
        let value = self.get(key)?;
        value.parse().ok()
    }

    /// 获取 u64 类型配置
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        let value = self.get(key)?;
        value.parse().ok()
    }

    /// 获取带默认值的配置
    pub fn get_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    /// 获取带默认值的 bool 配置
    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.get_bool(key).unwrap_or(default)
    }

    /// 获取带默认值的 i64 配置
    pub fn get_int_or(&self, key: &str, default: i64) -> i64 {
        self.get_int(key).unwrap_or(default)
    }

    /// 获取带默认值的 usize 配置
    pub fn get_usize_or(&self, key: &str, default: usize) -> usize {
        self.get_usize(key).unwrap_or(default)
    }

    /// 获取带默认值的 u64 配置
    pub fn get_u64_or(&self, key: &str, default: u64) -> u64 {
        self.get_u64(key).unwrap_or(default)
    }

    /// 设置配置值（同时更新数据库、内部缓存和 AppConfig）
    pub async fn set(&self, key: &str, value: &str) -> Result<ConfigUpdateResult> {
        // 先更新数据库
        let result = self.store.set(key, value).await?;

        // 更新内部缓存
        if let Ok(mut cache) = self.cache.write() {
            if let Some(item) = cache.get_mut(key) {
                item.value = std::sync::Arc::new(value.to_string());
                item.updated_at = chrono::Utc::now();
            }
        } else {
            warn!(
                "Cannot acquire runtime config cache write lock, skipping cache update for key: {}",
                key
            );
        }

        // 如果不需要重启，同步更新 AppConfig
        if !result.requires_restart {
            update_config_by_key(key, value);
        }

        Ok(result)
    }

    /// 获取配置变更历史
    pub async fn get_history(&self, key: &str, limit: u64) -> Result<Vec<ConfigHistoryEntry>> {
        self.store.get_history(key, limit).await
    }

    /// 获取底层存储（用于迁移等高级操作）
    pub fn store(&self) -> Arc<ConfigStore> {
        self.store.clone()
    }
}

/// 初始化全局运行时配置
///
/// 必须在数据库迁移完成后调用
pub async fn init_runtime_config(db: DatabaseConnection) -> Result<()> {
    let config = RuntimeConfig::new(db);
    config.load().await?;

    RUNTIME_CONFIG.set(config).map_err(|_| {
        ShortlinkerError::database_operation(
            "Runtime configuration already initialized".to_string(),
        )
    })?;

    Ok(())
}

/// 获取全局运行时配置
///
/// # Panics
/// 如果运行时配置未初始化，将会 panic
pub fn get_runtime_config() -> &'static RuntimeConfig {
    RUNTIME_CONFIG
        .get()
        .expect("Runtime configuration not initialized, please call init_runtime_config() first")
}

/// 尝试获取全局运行时配置
///
/// 如果运行时配置未初始化，返回 None
pub fn try_get_runtime_config() -> Option<&'static RuntimeConfig> {
    RUNTIME_CONFIG.get()
}

/// 配置键常量
pub mod keys {
    // API 认证
    pub const API_ADMIN_TOKEN: &str = "api.admin_token";
    pub const API_HEALTH_TOKEN: &str = "api.health_token";
    pub const API_JWT_SECRET: &str = "api.jwt_secret";
    pub const API_ACCESS_TOKEN_MINUTES: &str = "api.access_token_minutes";
    pub const API_REFRESH_TOKEN_DAYS: &str = "api.refresh_token_days";

    // 功能配置
    pub const FEATURES_RANDOM_CODE_LENGTH: &str = "features.random_code_length";
    pub const FEATURES_DEFAULT_URL: &str = "features.default_url";
    pub const FEATURES_ENABLE_ADMIN_PANEL: &str = "features.enable_admin_panel";

    // 点击统计
    pub const CLICK_ENABLE_TRACKING: &str = "click.enable_tracking";
    pub const CLICK_FLUSH_INTERVAL: &str = "click.flush_interval";

    // 路由配置
    pub const ROUTES_ADMIN_PREFIX: &str = "routes.admin_prefix";
    pub const ROUTES_HEALTH_PREFIX: &str = "routes.health_prefix";
    pub const ROUTES_FRONTEND_PREFIX: &str = "routes.frontend_prefix";

    // CORS 配置
    pub const CORS_ENABLED: &str = "cors.enabled";
    pub const CORS_ALLOWED_ORIGINS: &str = "cors.allowed_origins";
    pub const CORS_ALLOWED_METHODS: &str = "cors.allowed_methods";
    pub const CORS_ALLOWED_HEADERS: &str = "cors.allowed_headers";
    pub const CORS_MAX_AGE: &str = "cors.max_age";
    pub const CORS_ALLOW_CREDENTIALS: &str = "cors.allow_credentials";
}
