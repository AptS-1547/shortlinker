//! 配置管理 API 端点

use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use ts_rs::TS;

use crate::config::definitions::get_def;
use crate::config::types::ActionType;
use crate::config::{RuntimeConfig, get_all_schemas, try_get_runtime_config};
use crate::errors::ShortlinkerError;
use crate::system::reload::{ReloadTarget, get_reload_coordinator};

use super::helpers::{error_from_shortlinker, success_response};
use super::types::{ReloadResponse, TS_EXPORT_PATH, ValueType};

/// 获取 RuntimeConfig 或返回错误响应
///
/// 用于简化各 handler 中的重复模式。
/// 由于 actix-web handler 返回类型的限制，需要使用 `match` 而非 `?`。
fn require_runtime_config() -> Option<&'static RuntimeConfig> {
    try_get_runtime_config()
}

/// 返回 RuntimeConfig 未初始化的错误响应
fn runtime_config_unavailable_error() -> HttpResponse {
    error_from_shortlinker(&ShortlinkerError::service_unavailable(
        "Runtime config not initialized",
    ))
}

/// 宏：获取 RuntimeConfig 或返回错误
///
/// 由于 actix-web 的 handler 返回 `ActixResult<impl Responder>`，
/// 无法直接用 `?` 运算符返回 HttpResponse。此宏简化这种模式。
macro_rules! get_runtime_config_or_return {
    () => {
        match require_runtime_config() {
            Some(rc) => rc,
            None => return Ok(runtime_config_unavailable_error()),
        }
    };
}

/// 配置项响应
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct ConfigItemResponse {
    pub key: String,
    pub value: String,
    /// 值类型
    pub value_type: ValueType,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    pub updated_at: String,
}

/// 配置更新请求
#[derive(Debug, Deserialize, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct ConfigUpdateRequest {
    pub value: String,
}

/// 配置更新响应
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct ConfigUpdateResponse {
    pub key: String,
    pub value: String,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    pub message: Option<String>,
}

/// 配置历史记录响应
#[derive(Debug, Serialize, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct ConfigHistoryResponse {
    pub id: i32,
    pub config_key: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub changed_at: String,
    pub changed_by: Option<String>,
}

/// 获取所有配置
pub async fn get_all_configs(_req: HttpRequest) -> ActixResult<impl Responder> {
    let rc = get_runtime_config_or_return!();

    let configs = rc.get_all();
    let items: Vec<ConfigItemResponse> = configs
        .into_values()
        .map(|item| {
            // 对敏感配置屏蔽值
            let value = if item.is_sensitive {
                "[REDACTED]".to_string()
            } else {
                (*item.value).clone()
            };
            ConfigItemResponse {
                key: item.key,
                value,
                value_type: item.value_type,
                requires_restart: item.requires_restart,
                is_sensitive: item.is_sensitive,
                updated_at: item.updated_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(success_response(items))
}

/// 获取单个配置
pub async fn get_config(_req: HttpRequest, path: web::Path<String>) -> ActixResult<impl Responder> {
    let key = path.into_inner();
    let rc = get_runtime_config_or_return!();

    match rc.get_full(&key) {
        Some(item) => {
            // 对敏感配置屏蔽值
            let value = if item.is_sensitive {
                "[REDACTED]".to_string()
            } else {
                (*item.value).clone()
            };
            Ok(success_response(ConfigItemResponse {
                key: item.key,
                value,
                value_type: item.value_type,
                requires_restart: item.requires_restart,
                is_sensitive: item.is_sensitive,
                updated_at: item.updated_at.to_rfc3339(),
            }))
        }
        None => Ok(error_from_shortlinker(&ShortlinkerError::config_not_found(
            format!("Config key '{}' not found", key),
        ))),
    }
}

/// 更新配置
pub async fn update_config(
    _req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ConfigUpdateRequest>,
) -> ActixResult<impl Responder> {
    let key = path.into_inner();
    let rc = get_runtime_config_or_return!();

    match rc.set(&key, &body.value).await {
        Ok(result) => {
            // 敏感配置不记录明文值
            if result.is_sensitive {
                info!("Config updated: {} = [REDACTED]", key);
            } else {
                info!("Config updated: {} = {}", key, body.value);
            }

            let message = if result.requires_restart {
                Some("This configuration requires a service restart to take effect".to_string())
            } else {
                None
            };

            // 对敏感配置屏蔽返回值
            let value = if result.is_sensitive {
                "[REDACTED]".to_string()
            } else {
                result.value
            };

            Ok(success_response(ConfigUpdateResponse {
                key: result.key,
                value,
                requires_restart: result.requires_restart,
                is_sensitive: result.is_sensitive,
                message,
            }))
        }
        Err(e) => {
            warn!("Failed to update config {}: {}", key, e);
            Ok(error_from_shortlinker(
                &ShortlinkerError::config_update_failed(format!("Failed to update config: {}", e)),
            ))
        }
    }
}

/// 获取配置变更历史
pub async fn get_config_history(
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<HistoryQuery>,
) -> ActixResult<impl Responder> {
    let key = path.into_inner();
    let rc = get_runtime_config_or_return!();

    let limit = query.limit.unwrap_or(20).min(100);

    match rc.get_history(&key, limit).await {
        Ok(history) => {
            // 检查是否是敏感配置
            let is_sensitive = rc
                .get_full(&key)
                .map(|item| item.is_sensitive)
                .unwrap_or(false);

            let items: Vec<ConfigHistoryResponse> = history
                .into_iter()
                .map(|h| {
                    // 对敏感配置屏蔽历史值
                    let (old_value, new_value) = if is_sensitive {
                        (
                            h.old_value.map(|_| "[REDACTED]".to_string()),
                            "[REDACTED]".to_string(),
                        )
                    } else {
                        (h.old_value, h.new_value)
                    };
                    ConfigHistoryResponse {
                        id: h.id,
                        config_key: h.config_key,
                        old_value,
                        new_value,
                        changed_at: h.changed_at.to_rfc3339(),
                        changed_by: h.changed_by,
                    }
                })
                .collect();

            Ok(success_response(items))
        }
        Err(e) => {
            warn!("Failed to get config history for {}: {}", key, e);
            Ok(error_from_shortlinker(&ShortlinkerError::internal_error(
                format!("Failed to get config history: {}", e),
            )))
        }
    }
}

/// 重新加载配置
pub async fn reload_config(_req: HttpRequest) -> ActixResult<impl Responder> {
    // 使用 ReloadCoordinator 统一入口，确保与 CLI 走同一条路径
    let coordinator = match get_reload_coordinator() {
        Some(c) => c,
        None => {
            return Ok(error_from_shortlinker(
                &ShortlinkerError::service_unavailable("ReloadCoordinator not initialized"),
            ));
        }
    };

    match coordinator.reload(ReloadTarget::Config).await {
        Ok(result) => {
            info!("Config reloaded successfully in {}ms", result.duration_ms);
            Ok(success_response(ReloadResponse {
                message: "Config reloaded successfully".to_string(),
                duration_ms: result.duration_ms,
            }))
        }
        Err(e) => {
            warn!("Failed to reload config: {}", e);
            Ok(error_from_shortlinker(
                &ShortlinkerError::config_reload_failed(format!("Failed to reload config: {}", e)),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<u64>,
}

/// 获取所有配置的 schema
///
/// 返回配置项的元信息，包括类型、默认值、枚举选项等。
/// 前端用这个动态渲染配置表单。
pub async fn get_config_schema(_req: HttpRequest) -> ActixResult<impl Responder> {
    let schemas = get_all_schemas().clone();
    Ok(success_response(schemas))
}

// ========== Config Action API ==========

/// 配置 action 执行请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct ConfigActionRequest {
    pub action: ActionType,
}

/// 配置 action 执行响应
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct ConfigActionResponse {
    pub value: String,
}

/// 执行并保存响应（安全版本，不返回密钥值）
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct ExecuteAndSaveResponse {
    pub success: bool,
    pub requires_restart: bool,
    pub message: Option<String>,
}

/// 执行配置 action（如生成 token）
///
/// POST /admin/v1/config/{key}/action
pub async fn execute_config_action(
    _req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ConfigActionRequest>,
) -> ActixResult<impl Responder> {
    let key = path.into_inner();

    // 验证 key 存在且支持该 action
    let def = match get_def(&key) {
        Some(d) => d,
        None => {
            return Ok(error_from_shortlinker(
                &ShortlinkerError::config_not_found(format!("Config key '{}' not found", key)),
            ))
        }
    };

    // 检查是否支持请求的 action
    match def.action {
        Some(expected) if expected == body.action => {
            // 执行 action
            let value = execute_action(body.action);
            info!("Config action {:?} executed for key: {}", body.action, key);
            Ok(success_response(ConfigActionResponse { value }))
        }
        Some(_) => Ok(error_from_shortlinker(&ShortlinkerError::validation(
            format!(
                "Action {:?} not supported for config '{}', expected {:?}",
                body.action, key, def.action
            ),
        ))),
        None => Ok(error_from_shortlinker(&ShortlinkerError::validation(
            format!("Config '{}' does not support any action", key),
        ))),
    }
}

/// 执行具体的 action
fn execute_action(action: ActionType) -> String {
    match action {
        ActionType::GenerateToken => crate::utils::generate_secure_token(32),
    }
}

/// 执行配置 action 并保存（安全版本）
///
/// POST /admin/v1/config/{key}/execute-and-save
///
/// 密钥值在后端生成并保存，不返回给前端，最大化安全性。
pub async fn execute_and_save_config_action(
    _req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ConfigActionRequest>,
) -> ActixResult<impl Responder> {
    let key = path.into_inner();
    let rc = get_runtime_config_or_return!();

    // 验证 key 存在且支持该 action
    let def = match get_def(&key) {
        Some(d) => d,
        None => {
            return Ok(error_from_shortlinker(
                &ShortlinkerError::config_not_found(format!("Config key '{}' not found", key)),
            ))
        }
    };

    // 检查是否支持请求的 action
    match def.action {
        Some(expected) if expected == body.action => {
            // 执行 action 生成值
            let value = execute_action(body.action);

            // 保存值（密钥不返回给前端）
            match rc.set(&key, &value).await {
                Ok(result) => {
                    // 敏感配置不记录明文值
                    info!(
                        "Config '{}' action {:?} executed and saved (value redacted)",
                        key, body.action
                    );

                    let message = if result.requires_restart {
                        Some("Configuration saved. Server restart required to take effect.".to_string())
                    } else {
                        Some("Configuration saved successfully.".to_string())
                    };

                    Ok(success_response(ExecuteAndSaveResponse {
                        success: true,
                        requires_restart: result.requires_restart,
                        message,
                    }))
                }
                Err(e) => {
                    warn!("Failed to save config '{}': {}", key, e);
                    Ok(error_from_shortlinker(&ShortlinkerError::config_update_failed(
                        format!("Failed to save config: {}", e),
                    )))
                }
            }
        }
        Some(_) => Ok(error_from_shortlinker(&ShortlinkerError::validation(
            format!(
                "Action {:?} not supported for config '{}', expected {:?}",
                body.action, key, def.action
            ),
        ))),
        None => Ok(error_from_shortlinker(&ShortlinkerError::validation(
            format!("Config '{}' does not support any action", key),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ts_rs::TS;

    #[test]
    fn export_config_types() {
        let cfg = ts_rs::Config::default();
        ValueType::export_all(&cfg).expect("Failed to export ValueType");
        ConfigItemResponse::export_all(&cfg).expect("Failed to export ConfigItemResponse");
        ConfigUpdateRequest::export_all(&cfg).expect("Failed to export ConfigUpdateRequest");
        ConfigUpdateResponse::export_all(&cfg).expect("Failed to export ConfigUpdateResponse");
        ConfigHistoryResponse::export_all(&cfg).expect("Failed to export ConfigHistoryResponse");
        ConfigActionRequest::export_all(&cfg).expect("Failed to export ConfigActionRequest");
        ConfigActionResponse::export_all(&cfg).expect("Failed to export ConfigActionResponse");
        ExecuteAndSaveResponse::export_all(&cfg).expect("Failed to export ExecuteAndSaveResponse");
        println!("Config TypeScript types exported");
    }
}
