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
    /// 配置 key，与 registry 注册名一致（点分命名，如 `api.jwt_secret`）。
    pub key: String,
    /// 当前值的字符串形式；敏感配置经 `to_string` 后仍是原始值，
    /// 脱敏在上层 service 完成，Arc 用于跨任务共享避免拷贝。
    pub value: Arc<String>,
    /// 值的类型分类（string/number/bool/enum 等），决定解析与校验方式。
    pub value_type: ValueType,
    /// 修改后是否需要重启进程才能生效。
    pub requires_restart: bool,
    /// 是否为敏感配置；展示与导出时需要脱敏。
    pub is_sensitive: bool,
    /// 最近一次更新的时间（UTC）。
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
    /// 被更新的配置 key。
    pub key: String,
    /// 更新后的值（原始形式，未脱敏）。
    pub value: String,
    /// 该配置是否需要重启才能生效。
    pub requires_restart: bool,
    /// 该配置是否为敏感配置。
    pub is_sensitive: bool,
    /// 更新前的值；`None` 表示该 key 之前不存在。
    pub old_value: Option<String>,
}

/// 配置变更历史记录
#[derive(Debug, Clone)]
pub struct ConfigHistoryEntry {
    /// 历史记录主键，按时间递增。
    pub id: i32,
    /// 发生变更的配置 key。
    pub config_key: String,
    /// 变更前的值；`None` 表示首次写入。
    pub old_value: Option<String>,
    /// 变更后的值。
    pub new_value: String,
    /// 变更时间（UTC）。
    pub changed_at: chrono::DateTime<chrono::Utc>,
    /// 操作者标识；`None` 表示未记录。
    pub changed_by: Option<String>,
}

/// 配置存储服务
pub struct ConfigStore {
    db: DatabaseConnection,
}

impl ConfigStore {
    /// 基于已有数据库连接创建配置存储。
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
                        .upsert_prevalidated(
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
                        id: sea_orm::ActiveValue::default(),
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
