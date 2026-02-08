use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, Set, TransactionTrait, sea_query::OnConflict,
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
                ShortlinkerError::database_operation(format!(
                    "Failed to query config '{}': {}",
                    key, e
                ))
            })?;

        Ok(result.map(|m| m.value))
    }

    /// 获取单个配置的完整信息
    pub async fn get_full(&self, key: &str) -> Result<Option<ConfigItem>> {
        let result = system_config::Entity::find_by_id(key)
            .one(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!(
                    "Failed to query config '{}': {}",
                    key, e
                ))
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
                    "Failed to parse config '{}' value '{}'",
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
                ShortlinkerError::database_operation(format!(
                    "Failed to query config '{}': {}",
                    key, e
                ))
            })?;

        let (old_value, requires_restart, is_sensitive) = match &old_record {
            Some(r) => (Some(r.value.clone()), r.requires_restart, r.is_sensitive),
            None => {
                return Err(ShortlinkerError::database_operation(format!(
                    "Config key '{}' does not exist",
                    key
                )));
            }
        };

        // 如果值没变，直接返回
        if old_value.as_deref() == Some(value) {
            return Ok(ConfigUpdateResult {
                key: key.to_string(),
                value: value.to_string(),
                requires_restart,
                is_sensitive,
                old_value,
            });
        }

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

        // 使用事务确保更新和历史记录的原子性
        let txn = self.db.begin().await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to begin transaction: {}", e))
        })?;

        // 更新配置
        let mut active_model: system_config::ActiveModel = old_record.unwrap().into();
        active_model.value = Set(value.to_string());
        active_model.updated_at = Set(chrono::Utc::now());

        active_model.update(&txn).await.map_err(|e| {
            ShortlinkerError::database_operation(format!(
                "Failed to update config '{}': {}",
                key, e
            ))
        })?;

        let history = config_history::ActiveModel {
            id: Default::default(),
            config_key: Set(key.to_string()),
            old_value: Set(history_old_value),
            new_value: Set(history_new_value),
            changed_at: Set(chrono::Utc::now()),
            changed_by: Set(None),
        };

        history.insert(&txn).await.map_err(|e| {
            ShortlinkerError::database_operation(format!(
                "Failed to record config change history: {}",
                e
            ))
        })?;

        txn.commit().await.map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to commit transaction: {}", e))
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
                ShortlinkerError::database_operation(format!("Failed to query all configs: {}", e))
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

    /// 检查配置是否存在
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let count = system_config::Entity::find_by_id(key)
            .count(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!(
                    "Failed to check config '{}': {}",
                    key, e
                ))
            })?;

        Ok(count > 0)
    }

    /// 原子性插入配置（如果不存在）
    ///
    /// 使用 `INSERT ... ON CONFLICT DO NOTHING` 避免 TOCTOU 竞态条件。
    ///
    /// 返回值:
    /// - `Ok(true)`: 插入成功（配置之前不存在）
    /// - `Ok(false)`: 配置已存在，未执行插入
    pub async fn insert_if_not_exists(
        &self,
        key: &str,
        value: &str,
        value_type: ValueType,
        requires_restart: bool,
        is_sensitive: bool,
    ) -> Result<bool> {
        let model = system_config::ActiveModel {
            key: Set(key.to_string()),
            value: Set(value.to_string()),
            value_type: Set(value_type.to_string()),
            requires_restart: Set(requires_restart),
            is_sensitive: Set(is_sensitive),
            updated_at: Set(chrono::Utc::now()),
        };

        // 使用 ON CONFLICT DO NOTHING 实现原子性的 "insert if not exists"
        let result = system_config::Entity::insert(model)
            .on_conflict(
                OnConflict::column(system_config::Column::Key)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(&self.db)
            .await;

        match result {
            Ok(_) => Ok(true),
            Err(sea_orm::DbErr::RecordNotInserted) => Ok(false), // PostgreSQL
            Err(e) => {
                // 某些数据库后端在 do_nothing 时可能返回特定错误
                let err_str = e.to_string().to_lowercase();
                if err_str.contains("no rows") || err_str.contains("record not inserted") {
                    Ok(false)
                } else {
                    Err(ShortlinkerError::database_operation(format!(
                        "Failed to insert config '{}': {}",
                        key, e
                    )))
                }
            }
        }
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
                ShortlinkerError::database_operation(format!(
                    "Failed to query config '{}': {}",
                    key, e
                ))
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
            ShortlinkerError::database_operation(format!(
                "Failed to sync metadata for config '{}': {}",
                key, e
            ))
        })?;

        Ok(true)
    }

    /// 获取配置表中的记录数
    pub async fn count(&self) -> Result<u64> {
        let count = system_config::Entity::find()
            .count(&self.db)
            .await
            .map_err(|e| {
                ShortlinkerError::database_operation(format!("Failed to count configs: {}", e))
            })?;

        Ok(count)
    }

    /// 确保所有配置项存在,不存在则使用默认值初始化
    ///
    /// 遍历 `definitions::ALL_CONFIGS` 中定义的所有配置项，
    /// 对于数据库中不存在的配置，调用其 `default_fn` 生成默认值并插入。
    /// 同时同步配置的元数据（value_type, requires_restart, is_sensitive）。
    ///
    /// 特殊处理：
    /// - `api.admin_token`: 如果是新生成的明文密码，会写入 `admin_token.txt` 让用户保存，
    ///   然后哈希后再存入数据库。
    ///
    /// 应在服务启动时调用，确保首次启动时配置表不为空。
    pub async fn ensure_defaults(&self) -> Result<usize> {
        use crate::config::definitions::{ALL_CONFIGS, keys};
        use crate::utils::password::{is_argon2_hash, process_imported_password};
        use tracing::{debug, info, warn};

        let mut inserted_count = 0;

        for def in ALL_CONFIGS {
            // 调用默认值函数生成值
            let default_value = (def.default_fn)();

            // 特殊处理：api.admin_token 需要哈希
            let is_new_admin_token = def.key == keys::API_ADMIN_TOKEN
                && !default_value.is_empty()
                && !is_argon2_hash(&default_value);

            // 如果是 admin_token，通过 process_imported_password 处理（已哈希则保留，明文则哈希）
            let value_to_insert = if def.key == keys::API_ADMIN_TOKEN && !default_value.is_empty() {
                process_imported_password(Some(&default_value))
                    .map_err(|e| {
                        ShortlinkerError::database_operation(format!(
                            "Failed to hash admin_token: {}",
                            e
                        ))
                    })?
                    .unwrap_or_default()
            } else {
                default_value.clone()
            };

            // 尝试插入（如果不存在）
            let inserted = self
                .insert_if_not_exists(
                    def.key,
                    &value_to_insert,
                    def.value_type,
                    def.requires_restart,
                    def.is_sensitive,
                )
                .await?;

            if inserted {
                debug!("Initialized config '{}' with default value", def.key);
                inserted_count += 1;

                // 如果是新生成的 admin_token，写入明文到文件（可选操作，失败不影响安全性）
                if is_new_admin_token {
                    let token_file = std::path::Path::new("admin_token.txt");

                    // 安全修复：使用 create_new 防止 symlink 攻击
                    use std::fs::OpenOptions;
                    use std::io::Write;

                    match OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(token_file)
                    {
                        Ok(mut file) => {
                            // 设置文件权限 0600 (仅 Unix)
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                let perms = std::fs::Permissions::from_mode(0o600);
                                if let Err(e) = std::fs::set_permissions(token_file, perms) {
                                    warn!("Failed to set permissions on admin_token.txt: {}", e);
                                }
                            }

                            if let Err(e) = writeln!(
                                file,
                                "Auto-generated ADMIN_TOKEN (delete this file after saving):\n{}",
                                default_value
                            ) {
                                warn!(
                                    "Failed to write admin_token.txt: {}, but token is already hashed in database",
                                    e
                                );
                            } else {
                                info!(
                                    "Auto-generated admin token saved to admin_token.txt - please save it and delete the file"
                                );
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                            // 文件已存在，跳过
                            debug!("admin_token.txt already exists, skipping");
                        }
                        Err(e) => {
                            warn!(
                                "Failed to create admin_token.txt: {}, but token is already hashed in database",
                                e
                            );
                        }
                    }
                }
            } else {
                // 配置已存在，同步元数据
                if self
                    .sync_metadata(
                        def.key,
                        def.value_type,
                        def.requires_restart,
                        def.is_sensitive,
                    )
                    .await?
                {
                    debug!("Synced metadata for config '{}'", def.key);
                }
            }
        }

        if inserted_count > 0 {
            info!("Initialized {} default configuration items", inserted_count);
        }

        Ok(inserted_count)
    }
}
