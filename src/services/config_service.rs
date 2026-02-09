//! Configuration management service
//!
//! Provides unified business logic for runtime configuration operations.
//! Encapsulates sensitive value redaction, action execution, and config reload.

use tracing::info;

use crate::config::definitions::get_def;
use crate::config::types::ActionType;
use crate::config::{RuntimeConfig, ValueType, get_all_schemas, try_get_runtime_config};
use crate::errors::ShortlinkerError;
use crate::storage::{ConfigItem, ConfigUpdateResult};
use crate::system::reload::{ReloadTarget, get_reload_coordinator};

const REDACTED: &str = "[REDACTED]";

// ============ Service DTOs ============

/// 配置项视图（敏感值已屏蔽）
#[derive(Debug, Clone)]
pub struct ConfigItemView {
    pub key: String,
    pub value: String,
    pub value_type: ValueType,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// 配置更新结果视图（敏感值已屏蔽）
#[derive(Debug, Clone)]
pub struct ConfigUpdateView {
    pub key: String,
    pub value: String,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    pub message: Option<String>,
}

/// 配置历史记录视图（敏感值已屏蔽）
#[derive(Debug, Clone)]
pub struct ConfigHistoryView {
    pub id: i32,
    pub config_key: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub changed_by: Option<String>,
}

/// 配置 reload 结果
#[derive(Debug, Clone)]
pub struct ConfigReloadResult {
    pub duration_ms: u64,
}

// ============ ConfigService Implementation ============

/// Service for runtime configuration management
///
/// Encapsulates RuntimeConfig operations with consistent sensitive value redaction.
/// All config read operations automatically redact sensitive values.
pub struct ConfigService {
    runtime_config: &'static RuntimeConfig,
}

impl ConfigService {
    /// Create a new ConfigService
    ///
    /// Returns error if RuntimeConfig is not yet initialized.
    pub fn new() -> Result<Self, ShortlinkerError> {
        let rc = try_get_runtime_config().ok_or_else(|| {
            ShortlinkerError::service_unavailable("Runtime config not initialized")
        })?;
        Ok(Self { runtime_config: rc })
    }

    /// 将 ConfigItem 转换为 ConfigItemView（屏蔽敏感值）
    fn to_item_view(item: ConfigItem) -> ConfigItemView {
        let value = if item.is_sensitive {
            REDACTED.to_string()
        } else {
            (*item.value).clone()
        };
        ConfigItemView {
            key: item.key,
            value,
            value_type: item.value_type,
            requires_restart: item.requires_restart,
            is_sensitive: item.is_sensitive,
            updated_at: item.updated_at,
        }
    }

    /// 将 ConfigUpdateResult 转换为 ConfigUpdateView（屏蔽敏感值）
    fn to_update_view(result: ConfigUpdateResult) -> ConfigUpdateView {
        let value = if result.is_sensitive {
            REDACTED.to_string()
        } else {
            result.value
        };
        let message = if result.requires_restart {
            Some("This configuration requires a service restart to take effect".to_string())
        } else {
            None
        };
        ConfigUpdateView {
            key: result.key,
            value,
            requires_restart: result.requires_restart,
            is_sensitive: result.is_sensitive,
            message,
        }
    }

    /// 获取所有配置（敏感值已屏蔽）
    pub fn get_all(&self) -> Vec<ConfigItemView> {
        self.runtime_config
            .get_all()
            .into_values()
            .map(Self::to_item_view)
            .collect()
    }

    /// 获取单个配置（敏感值已屏蔽）
    pub fn get(&self, key: &str) -> Result<ConfigItemView, ShortlinkerError> {
        self.runtime_config
            .get_full(key)
            .map(Self::to_item_view)
            .ok_or_else(|| {
                ShortlinkerError::config_not_found(format!("Config key '{}' not found", key))
            })
    }

    /// 更新配置
    pub async fn update(
        &self,
        key: &str,
        value: &str,
    ) -> Result<ConfigUpdateView, ShortlinkerError> {
        let result = self.runtime_config.set(key, value).await?;

        if result.is_sensitive {
            info!("Config updated: {} = {}", key, REDACTED);
        } else {
            info!("Config updated: {} = {}", key, value);
        }

        Ok(Self::to_update_view(result))
    }

    /// 获取配置变更历史（敏感值已屏蔽）
    pub async fn get_history(
        &self,
        key: &str,
        limit: u64,
    ) -> Result<Vec<ConfigHistoryView>, ShortlinkerError> {
        let history = self.runtime_config.get_history(key, limit).await?;

        let is_sensitive = self
            .runtime_config
            .get_full(key)
            .map(|item| item.is_sensitive)
            .unwrap_or(false);

        Ok(history
            .into_iter()
            .map(|h| {
                let (old_value, new_value) = if is_sensitive {
                    (
                        h.old_value.map(|_| REDACTED.to_string()),
                        REDACTED.to_string(),
                    )
                } else {
                    (h.old_value, h.new_value)
                };
                ConfigHistoryView {
                    id: h.id,
                    config_key: h.config_key,
                    old_value,
                    new_value,
                    changed_at: h.changed_at,
                    changed_by: h.changed_by,
                }
            })
            .collect())
    }

    /// 获取配置 schema
    pub fn get_schema(&self) -> Vec<crate::config::schema::ConfigSchema> {
        get_all_schemas().clone()
    }

    /// 执行配置 action（如生成 token）
    pub fn execute_action(
        &self,
        key: &str,
        action: ActionType,
    ) -> Result<String, ShortlinkerError> {
        let def = get_def(key).ok_or_else(|| {
            ShortlinkerError::config_not_found(format!("Config key '{}' not found", key))
        })?;

        match def.action {
            Some(expected) if expected == action => {
                let value = Self::run_action(action);
                info!("Config action {:?} executed for key: {}", action, key);
                Ok(value)
            }
            Some(_) => Err(ShortlinkerError::validation(format!(
                "Action {:?} not supported for config '{}', expected {:?}",
                action, key, def.action
            ))),
            None => Err(ShortlinkerError::validation(format!(
                "Config '{}' does not support any action",
                key
            ))),
        }
    }

    /// 执行 action 并保存（安全版本，不返回值）
    pub async fn execute_and_save(
        &self,
        key: &str,
        action: ActionType,
    ) -> Result<ConfigUpdateView, ShortlinkerError> {
        let def = get_def(key).ok_or_else(|| {
            ShortlinkerError::config_not_found(format!("Config key '{}' not found", key))
        })?;

        match def.action {
            Some(expected) if expected == action => {
                let value = Self::run_action(action);
                let result = self.runtime_config.set(key, &value).await.map_err(|e| {
                    ShortlinkerError::config_update_failed(format!("Failed to save config: {}", e))
                })?;

                info!(
                    "Config '{}' action {:?} executed and saved (value redacted)",
                    key, action
                );

                Ok(Self::to_update_view(result))
            }
            Some(_) => Err(ShortlinkerError::validation(format!(
                "Action {:?} not supported for config '{}', expected {:?}",
                action, key, def.action
            ))),
            None => Err(ShortlinkerError::validation(format!(
                "Config '{}' does not support any action",
                key
            ))),
        }
    }

    /// 重新加载配置
    pub async fn reload(&self) -> Result<ConfigReloadResult, ShortlinkerError> {
        let coordinator = get_reload_coordinator().ok_or_else(|| {
            ShortlinkerError::service_unavailable("ReloadCoordinator not initialized")
        })?;

        let result = coordinator
            .reload(ReloadTarget::Config)
            .await
            .map_err(|e| {
                ShortlinkerError::config_reload_failed(format!("Failed to reload config: {}", e))
            })?;

        info!("Config reloaded successfully in {}ms", result.duration_ms);

        Ok(ConfigReloadResult {
            duration_ms: result.duration_ms,
        })
    }

    fn run_action(action: ActionType) -> String {
        match action {
            ActionType::GenerateToken => crate::utils::generate_secure_token(32),
        }
    }
}
