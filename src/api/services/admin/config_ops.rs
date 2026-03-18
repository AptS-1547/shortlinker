//! 配置管理 API 端点

use actix_web::{HttpRequest, Responder, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;

use crate::services::ConfigService;

use super::helpers::{error_from_shortlinker, success_response};
use super::types::{ReloadResponse, TS_EXPORT_PATH, ValueType};

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

// ========== Config Action API types ==========

/// 配置 action 执行请求
#[derive(Debug, Deserialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct ConfigActionRequest {
    pub action: crate::config::types::ActionType,
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

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub limit: Option<u64>,
}

// ========== Handlers ==========

/// 获取所有配置
pub async fn get_all_configs(
    _req: HttpRequest,
    service: web::Data<Arc<ConfigService>>,
) -> ActixResult<impl Responder> {
    let items: Vec<ConfigItemResponse> = service
        .get_all()
        .into_iter()
        .map(|item| ConfigItemResponse {
            key: item.key,
            value: item.value,
            value_type: item.value_type,
            requires_restart: item.requires_restart,
            is_sensitive: item.is_sensitive,
            updated_at: item.updated_at.to_rfc3339(),
        })
        .collect();

    Ok(success_response(items))
}

/// 获取单个配置
pub async fn get_config(
    _req: HttpRequest,
    path: web::Path<String>,
    service: web::Data<Arc<ConfigService>>,
) -> ActixResult<impl Responder> {
    let key = path.into_inner();

    match service.get(&key) {
        Ok(item) => Ok(success_response(ConfigItemResponse {
            key: item.key,
            value: item.value,
            value_type: item.value_type,
            requires_restart: item.requires_restart,
            is_sensitive: item.is_sensitive,
            updated_at: item.updated_at.to_rfc3339(),
        })),
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// 更新配置
pub async fn update_config(
    _req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ConfigUpdateRequest>,
    service: web::Data<Arc<ConfigService>>,
) -> ActixResult<impl Responder> {
    let key = path.into_inner();

    match service.update(&key, &body.value).await {
        Ok(view) => Ok(success_response(ConfigUpdateResponse {
            key: view.key,
            value: view.value,
            requires_restart: view.requires_restart,
            is_sensitive: view.is_sensitive,
            message: view.message,
        })),
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// 获取配置变更历史
pub async fn get_config_history(
    _req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<HistoryQuery>,
    service: web::Data<Arc<ConfigService>>,
) -> ActixResult<impl Responder> {
    let key = path.into_inner();
    let limit = query.limit.unwrap_or(20).min(100);

    match service.get_history(&key, limit).await {
        Ok(history) => {
            let items: Vec<ConfigHistoryResponse> = history
                .into_iter()
                .map(|h| ConfigHistoryResponse {
                    id: h.id,
                    config_key: h.config_key,
                    old_value: h.old_value,
                    new_value: h.new_value,
                    changed_at: h.changed_at.to_rfc3339(),
                    changed_by: h.changed_by,
                })
                .collect();
            Ok(success_response(items))
        }
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// 重新加载配置
pub async fn reload_config(
    _req: HttpRequest,
    service: web::Data<Arc<ConfigService>>,
) -> ActixResult<impl Responder> {
    match service.reload().await {
        Ok(result) => Ok(success_response(ReloadResponse {
            message: "Config reloaded successfully".to_string(),
            duration_ms: result.duration_ms,
        })),
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// 获取所有配置的 schema
pub async fn get_config_schema(_req: HttpRequest) -> ActixResult<impl Responder> {
    let schemas = crate::config::get_all_schemas().clone();
    Ok(success_response(schemas))
}

/// 执行配置 action（如生成 token）
///
/// POST /admin/v1/config/{key}/action
pub async fn execute_config_action(
    _req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ConfigActionRequest>,
    service: web::Data<Arc<ConfigService>>,
) -> ActixResult<impl Responder> {
    let key = path.into_inner();

    match service.execute_action(&key, body.action) {
        Ok(value) => Ok(success_response(ConfigActionResponse { value })),
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// 执行配置 action 并保存（安全版本）
///
/// POST /admin/v1/config/{key}/execute-and-save
pub async fn execute_and_save_config_action(
    _req: HttpRequest,
    path: web::Path<String>,
    body: web::Json<ConfigActionRequest>,
    service: web::Data<Arc<ConfigService>>,
) -> ActixResult<impl Responder> {
    match service.execute_and_save(&path, body.action).await {
        Ok(view) => Ok(success_response(ExecuteAndSaveResponse {
            success: true,
            requires_restart: view.requires_restart,
            message: view.message,
        })),
        Err(e) => Ok(error_from_shortlinker(&e)),
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
