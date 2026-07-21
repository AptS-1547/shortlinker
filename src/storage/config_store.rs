use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use std::collections::HashMap;
use std::sync::Arc;

use aster_forge_db::system_config::{
    Model as ForgeSystemConfig, SystemConfigDbBinding, SystemConfigUpsert,
};

use crate::config::ValueType;
use crate::config::definitions::CONFIG_REGISTRY;
use crate::errors::{Result, ShortlinkerError};
use migration::entities::config_history;

pub(crate) static SYSTEM_CONFIG_BINDING: SystemConfigDbBinding =
    SystemConfigDbBinding::new(&CONFIG_REGISTRY, &[]);

/// 配置项的完整信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigItem {
    pub key: String,
    pub value: Arc<String>,
    pub value_type: ValueType,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl ConfigItem {
    fn from_model(model: ForgeSystemConfig) -> Self {
        Self {
            value_type: ValueType::from_forge(&model.key, model.value_type),
            key: model.key,
            value: Arc::new(model.value),
            requires_restart: model.requires_restart,
            is_sensitive: model.is_sensitive,
            updated_at: model.updated_at,
        }
    }
}

impl aster_forge_config::RuntimeConfigRecord for ConfigItem {
    fn config_key(&self) -> &str {
        &self.key
    }

    fn config_value(&self) -> &str {
        self.value.as_str()
    }

    fn config_requires_restart(&self) -> bool {
        self.requires_restart
    }
}

/// 配置更新结果
#[derive(Debug, Clone)]
pub struct ConfigUpdateResult {
    pub key: String,
    pub value: String,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    pub old_value: Option<String>,
}

/// 配置变更历史记录
#[derive(Debug, Clone)]
pub struct ConfigHistoryEntry {
    pub id: i32,
    pub config_key: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub changed_at: chrono::DateTime<chrono::Utc>,
    pub changed_by: Option<String>,
}

/// 配置存储服务
pub struct ConfigStore {
    db: DatabaseConnection,
}

impl ConfigStore {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// 设置配置值
    pub async fn set(&self, key: &str, value: &str) -> Result<ConfigUpdateResult> {
        let retry_config = aster_forge_db::retry::RetryConfig::deadlock();
        let key = key.to_string();
        let value = value.to_string();
        aster_forge_db::transaction::with_transaction_retry(
            &self.db,
            &retry_config,
            |txn| {
                let key = key.clone();
                let value = value.clone();
                Box::pin(async move {
                    SYSTEM_CONFIG_BINDING.lock_by_key(txn, &key).await?;
                    let old_record = SYSTEM_CONFIG_BINDING
                        .find_by_key(txn, &key)
                        .await?
                        .ok_or_else(|| {
                            aster_forge_db::DbError::non_retryable(format!(
                                "Config key '{}' does not exist",
                                key
                            ))
                        })?;
                    let old_value = old_record.value.clone();

                    if old_value == value {
                        return Ok(ConfigUpdateResult {
                            key,
                            value,
                            requires_restart: old_record.requires_restart,
                            is_sensitive: old_record.is_sensitive,
                            old_value: Some(old_value),
                        });
                    }

                    let updated = SYSTEM_CONFIG_BINDING
                        .upsert(
                            txn,
                            SystemConfigUpsert {
                                key: &key,
                                value: &value,
                                visibility: None,
                                updated_by: None,
                            },
                        )
                        .await?;
                    let is_sensitive = updated.is_sensitive;
                    let (history_old_value, history_new_value) = if is_sensitive {
                        (Some("[REDACTED]".to_string()), "[REDACTED]".to_string())
                    } else {
                        (Some(old_value.clone()), value)
                    };

                    config_history::ActiveModel {
                        id: Default::default(),
                        config_key: Set(key),
                        old_value: Set(history_old_value),
                        new_value: Set(history_new_value),
                        changed_at: Set(updated.updated_at),
                        changed_by: Set(None),
                    }
                    .insert(txn)
                    .await
                    .map_err(aster_forge_db::DbError::from)?;

                    Ok(ConfigUpdateResult {
                        key: updated.key,
                        value: updated.value,
                        requires_restart: updated.requires_restart,
                        is_sensitive,
                        old_value: Some(old_value),
                    })
                })
            },
            aster_forge_db::DbError::is_retryable,
        )
        .await
        .map_err(ShortlinkerError::from)
    }

    /// 获取所有配置
    pub async fn get_all(&self) -> Result<HashMap<String, ConfigItem>> {
        let records = SYSTEM_CONFIG_BINDING.find_all(&self.db).await?;

        let mut map = HashMap::new();
        for r in records {
            map.insert(r.key.clone(), ConfigItem::from_model(r));
        }

        Ok(map)
    }

    /// 获取配置变更历史
    pub async fn get_history(&self, key: &str, limit: u64) -> Result<Vec<ConfigHistoryEntry>> {
        let records = config_history::Entity::find()
            .filter(config_history::Column::ConfigKey.eq(key))
            .order_by_desc(config_history::Column::ChangedAt)
            .paginate(&self.db, limit)
            .fetch_page(0)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!(
                    "Failed to query config history: {}",
                    e
                ))
            })?;

        Ok(records
            .into_iter()
            .map(|r| ConfigHistoryEntry {
                id: r.id,
                config_key: r.config_key,
                old_value: r.old_value,
                new_value: r.new_value,
                changed_at: r.changed_at,
                changed_by: r.changed_by,
            })
            .collect())
    }
}
