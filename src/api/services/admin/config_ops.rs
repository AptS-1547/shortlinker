//! 配置管理 API 端点

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, Responder, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::config::try_get_runtime_config;

use super::helpers::{error_response, success_response};

/// 配置项响应
#[derive(Debug, Serialize)]
pub struct ConfigItemResponse {
    pub key: String,
    pub value: String,
    pub value_type: String,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    pub updated_at: String,
}

/// 配置更新请求
#[derive(Debug, Deserialize)]
pub struct ConfigUpdateRequest {
    pub value: String,
}

/// 配置更新响应
#[derive(Debug, Serialize)]
pub struct ConfigUpdateResponse {
    pub key: String,
    pub value: String,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    pub message: Option<String>,
}

/// 配置历史记录响应
#[derive(Debug, Serialize)]
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
    let rc = match try_get_runtime_config() {
        Some(rc) => rc,
        None => {
            return Ok(error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "Runtime config not initialized",
            ));
        }
    };

    let configs = rc.get_all();
    let items: Vec<ConfigItemResponse> = configs
        .into_values()
        .map(|item| {
            // 对敏感配置屏蔽值
            let value = if item.is_sensitive {
                "********".to_string()
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

    let rc = match try_get_runtime_config() {
        Some(rc) => rc,
        None => {
            return Ok(error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "Runtime config not initialized",
            ));
        }
    };

    match rc.get_full(&key) {
        Some(item) => {
            // 对敏感配置屏蔽值
            let value = if item.is_sensitive {
                "********".to_string()
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
        None => Ok(error_response(
            StatusCode::NOT_FOUND,
            &format!("Config key '{}' not found", key),
        )),
    }
}

/// 更新配置
pub async fn update_config(
    _req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ConfigUpdateRequest>,
) -> ActixResult<impl Responder> {
    let key = path.into_inner();

    let rc = match try_get_runtime_config() {
        Some(rc) => rc,
        None => {
            return Ok(error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "Runtime config not initialized",
            ));
        }
    };

    match rc.set(&key, &body.value).await {
        Ok(result) => {
            info!("Config updated: {} = {}", key, body.value);

            let message = if result.requires_restart {
                Some("此配置需要重启服务后生效".to_string())
            } else {
                None
            };

            // 对敏感配置屏蔽返回值
            let value = if result.is_sensitive {
                "********".to_string()
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
            Ok(error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to update config: {}", e),
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

    let rc = match try_get_runtime_config() {
        Some(rc) => rc,
        None => {
            return Ok(error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "Runtime config not initialized",
            ));
        }
    };

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
                            h.old_value.map(|_| "********".to_string()),
                            "********".to_string(),
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
            Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to get config history: {}", e),
            ))
        }
    }
}

/// 重新加载配置
pub async fn reload_config(_req: HttpRequest) -> ActixResult<impl Responder> {
    let rc = match try_get_runtime_config() {
        Some(rc) => rc,
        None => {
            return Ok(error_response(
                StatusCode::SERVICE_UNAVAILABLE,
                "Runtime config not initialized",
            ));
        }
    };

    match rc.reload().await {
        Ok(_) => {
            info!("Config reloaded successfully");
            Ok(success_response(serde_json::json!({
                "message": "Config reloaded successfully"
            })))
        }
        Err(e) => {
            warn!("Failed to reload config: {}", e);
            Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Failed to reload config: {}", e),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<u64>,
}
