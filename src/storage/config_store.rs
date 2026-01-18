use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set,
};
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::ValueType;
use crate::config::definitions::get_def;
use crate::errors::{Result, ShortlinkerError};
use migration::entities::{config_history, system_config};

/// 配置项的完整信息
#[derive(Debug, Clone)]
pub struct ConfigItem {
    pub key: String,
    pub value: Arc<String>,
    pub value_type: ValueType,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    pub updated_at: chrono::DateTime<chrono::Utc>,
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

    /// 获取单个配置值
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let result = system_config::Entity::find_by_id(key)
            .one(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("查询配置 '{}' 失败: {}", key, e))
            })?;

        Ok(result.map(|m| m.value))
    }

    /// 获取单个配置的完整信息
    pub async fn get_full(&self, key: &str) -> Result<Option<ConfigItem>> {
        let result = system_config::Entity::find_by_id(key)
            .one(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("查询配置 '{}' 失败: {}", key, e))
            })?;

        Ok(result.map(|m| ConfigItem {
            key: m.key,
            value: Arc::new(m.value),
            value_type: m.value_type.parse().unwrap_or(ValueType::String),
            requires_restart: m.requires_restart,
            is_sensitive: m.is_sensitive,
            updated_at: m.updated_at,
        }))
    }

    /// 获取配置并解析为指定类型
    pub async fn get_typed<T: std::str::FromStr>(&self, key: &str) -> Result<Option<T>> {
        let value = self.get(key).await?;
        match value {
            Some(v) => match v.parse::<T>() {
                Ok(parsed) => Ok(Some(parsed)),
                Err(_) => Err(ShortlinkerError::database_operation(format!(
                    "配置 '{}' 值 '{}' 解析失败",
                    key, v
                ))),
            },
            None => Ok(None),
        }
    }

    /// 获取 bool 类型配置
    pub async fn get_bool(&self, key: &str) -> Result<Option<bool>> {
        let value = self.get(key).await?;
        match value {
            Some(v) => {
                let v_lower = v.to_lowercase();
                Ok(Some(
                    v_lower == "true" || v_lower == "1" || v_lower == "yes",
                ))
            }
            None => Ok(None),
        }
    }

    /// 获取 i64 类型配置
    pub async fn get_int(&self, key: &str) -> Result<Option<i64>> {
        self.get_typed::<i64>(key).await
    }

    /// 设置配置值
    pub async fn set(&self, key: &str, value: &str) -> Result<ConfigUpdateResult> {
        // 先获取旧值和元信息
        let old_record = system_config::Entity::find_by_id(key)
            .one(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("查询配置 '{}' 失败: {}", key, e))
            })?;

        let (old_value, requires_restart, is_sensitive) = match &old_record {
            Some(r) => (Some(r.value.clone()), r.requires_restart, r.is_sensitive),
            None => {
                return Err(ShortlinkerError::database_operation(format!(
                    "配置项 '{}' 不存在",
                    key
                )));
            }
        };

        // 如果值没变，直接返回
        if old_value.as_ref() == Some(&value.to_string()) {
            return Ok(ConfigUpdateResult {
                key: key.to_string(),
                value: value.to_string(),
                requires_restart,
                is_sensitive,
                old_value,
            });
        }

        // 更新配置
        let mut active_model: system_config::ActiveModel = old_record.unwrap().into();
        active_model.value = Set(value.to_string());
        active_model.updated_at = Set(chrono::Utc::now());

        active_model.update(&self.db).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("更新配置 '{}' 失败: {}", key, e))
        })?;

        // 检查是否为敏感配置（优先使用定义，回退到数据库标记）
        let is_sensitive_config = get_def(key)
            .map(|def| def.is_sensitive)
            .unwrap_or(is_sensitive);

        // 记录变更历史（敏感配置的值脱敏为 [REDACTED]）
        let (history_old_value, history_new_value) = if is_sensitive_config {
            (
                old_value.as_ref().map(|_| "[REDACTED]".to_string()),
                "[REDACTED]".to_string(),
            )
        } else {
            (old_value.clone(), value.to_string())
        };

        let history = config_history::ActiveModel {
            id: Default::default(),
            config_key: Set(key.to_string()),
            old_value: Set(history_old_value),
            new_value: Set(history_new_value),
            changed_at: Set(chrono::Utc::now()),
            changed_by: Set(None), // TODO: 未来可以传入操作者信息
        };

        history.insert(&self.db).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("记录配置变更历史失败: {}", e))
        })?;

        Ok(ConfigUpdateResult {
            key: key.to_string(),
            value: value.to_string(),
            requires_restart,
            is_sensitive,
            old_value,
        })
    }

    /// 获取所有配置
    pub async fn get_all(&self) -> Result<HashMap<String, ConfigItem>> {
        let records = system_config::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("查询所有配置失败: {}", e))
            })?;

        let mut map = HashMap::new();
        for r in records {
            map.insert(
                r.key.clone(),
                ConfigItem {
                    key: r.key,
                    value: Arc::new(r.value),
                    value_type: r.value_type.parse().unwrap_or(ValueType::String),
                    requires_restart: r.requires_restart,
                    is_sensitive: r.is_sensitive,
                    updated_at: r.updated_at,
                },
            );
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
                ShortlinkerError::database_operation(format!("查询配置历史失败: {}", e))
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

    /// 检查配置是否存在
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let count = system_config::Entity::find_by_id(key)
            .count(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("检查配置 '{}' 失败: {}", key, e))
            })?;

        Ok(count > 0)
    }

    /// 批量插入配置
    pub async fn insert_if_not_exists(
        &self,
        key: &str,
        value: &str,
        value_type: ValueType,
        requires_restart: bool,
        is_sensitive: bool,
    ) -> Result<bool> {
        // 检查是否已存在
        if self.exists(key).await? {
            return Ok(false);
        }

        let model = system_config::ActiveModel {
            key: Set(key.to_string()),
            value: Set(value.to_string()),
            value_type: Set(value_type.to_string()),
            requires_restart: Set(requires_restart),
            is_sensitive: Set(is_sensitive),
            updated_at: Set(chrono::Utc::now()),
        };

        model.insert(&self.db).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("插入配置 '{}' 失败: {}", key, e))
        })?;

        Ok(true)
    }

    /// 同步配置项的元信息（不修改 value / updated_at）
    ///
    /// 用于配置定义变更后的兼容处理，确保数据库中的：
    /// - value_type
    /// - requires_restart
    /// - is_sensitive
    ///
    /// 与代码中的定义保持一致。
    ///
    /// 返回值表示是否发生了更新。
    pub async fn sync_metadata(
        &self,
        key: &str,
        value_type: ValueType,
        requires_restart: bool,
        is_sensitive: bool,
    ) -> Result<bool> {
        let record = system_config::Entity::find_by_id(key)
            .one(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("查询配置 '{}' 失败: {}", key, e))
            })?;

        let Some(r) = record else {
            return Ok(false);
        };

        let desired_value_type = value_type.to_string();
        let current_value_type = r.value_type.clone();
        let current_requires_restart = r.requires_restart;
        let current_is_sensitive = r.is_sensitive;

        let mut changed = false;
        let mut active_model: system_config::ActiveModel = r.into();

        if current_value_type != desired_value_type {
            active_model.value_type = Set(desired_value_type);
            changed = true;
        }

        if current_requires_restart != requires_restart {
            active_model.requires_restart = Set(requires_restart);
            changed = true;
        }

        if current_is_sensitive != is_sensitive {
            active_model.is_sensitive = Set(is_sensitive);
            changed = true;
        }

        if !changed {
            return Ok(false);
        }

        active_model.update(&self.db).await.map_err(|e| {
            ShortlinkerError::database_operation(format!("同步配置 '{}' 的元信息失败: {}", key, e))
        })?;

        Ok(true)
    }

    /// 获取配置表中的记录数
    pub async fn count(&self) -> Result<u64> {
        let count = system_config::Entity::find()
            .count(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("统计配置数量失败: {}", e))
            })?;

        Ok(count)
    }
}
