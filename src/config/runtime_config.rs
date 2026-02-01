use sea_orm::DatabaseConnection;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use tracing::info;

use crate::errors::{Result, ShortlinkerError};
use crate::storage::{ConfigHistoryEntry, ConfigItem, ConfigStore, ConfigUpdateResult};

use super::validators;

// Re-export keys from definitions module
pub use super::definitions::keys;

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

    /// 从数据库加载所有配置到缓存
    pub async fn load(&self) -> Result<()> {
        let configs = self.store.get_all().await?;
        let count = configs.len();

        // 更新内部缓存
        {
            let mut cache = self.cache.write().map_err(|_| {
                ShortlinkerError::internal_error("Runtime config cache lock poisoned".to_string())
            })?;
            *cache = configs;
        }

        info!("Loaded {} runtime configuration items", count);
        Ok(())
    }

    /// 运行时热重载配置
    pub async fn reload(&self) -> Result<()> {
        let configs = self.store.get_all().await?;
        let count = configs.len();

        // 更新内部缓存
        {
            let mut cache = self.cache.write().map_err(|_| {
                ShortlinkerError::internal_error("Runtime config cache lock poisoned".to_string())
            })?;
            *cache = configs;
        }

        info!("Reloaded {} runtime configuration items", count);
        Ok(())
    }

    /// 获取配置值
    pub fn get(&self, key: &str) -> Option<String> {
        let cache = self
            .cache
            .read()
            .expect("Runtime config cache lock poisoned - this is a fatal error");
        cache.get(key).map(|item| (*item.value).clone())
    }

    /// 获取配置的完整信息
    pub fn get_full(&self, key: &str) -> Option<ConfigItem> {
        let cache = self
            .cache
            .read()
            .expect("Runtime config cache lock poisoned - this is a fatal error");
        cache.get(key).cloned()
    }

    /// 获取所有配置
    pub fn get_all(&self) -> HashMap<String, ConfigItem> {
        self.cache
            .read()
            .expect("Runtime config cache lock poisoned - this is a fatal error")
            .clone()
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

    /// 设置配置值（同时更新数据库和内部缓存）
    ///
    /// 对于 `requires_restart=true` 的配置，只更新数据库，不更新内存缓存。
    /// 这确保在重启前，所有读取该配置的代码都获得一致的旧值。
    pub async fn set(&self, key: &str, value: &str) -> Result<ConfigUpdateResult> {
        // 验证 enum 类型配置值
        if let Err(e) = validators::validate_config_value(key, value) {
            return Err(ShortlinkerError::validation(format!(
                "Invalid value for '{}': {}",
                key, e
            )));
        }

        // 先更新数据库
        let result = self.store.set(key, value).await?;

        // 对于 requires_restart=true 的配置，不更新内存缓存
        // 这确保在重启前，所有读取该配置的代码都获得一致的旧值
        if result.requires_restart {
            return Ok(result);
        }

        // 更新内部缓存（必须成功，保证一致性）
        let mut cache = self.cache.write().map_err(|_| {
            ShortlinkerError::internal_error("Runtime config cache lock poisoned".to_string())
        })?;

        if let Some(item) = cache.get_mut(key) {
            item.value = std::sync::Arc::new(value.to_string());
            item.updated_at = chrono::Utc::now();
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
/// 必须在数据库迁移完成后调用。
///
/// 初始化流程：
/// 1. 确保所有配置项存在（首次启动时使用默认值初始化）
/// 2. 从数据库加载配置到内存缓存
pub async fn init_runtime_config(db: DatabaseConnection) -> Result<()> {
    let config = RuntimeConfig::new(db);

    // 确保所有配置项存在（首次启动时使用默认值初始化）
    config.store().ensure_defaults().await?;

    // 加载配置到内存缓存
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
