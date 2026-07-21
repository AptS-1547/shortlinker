//! 配置管理 API 端点

use actix_web::{HttpRequest, Responder, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::services::ConfigService;

use super::helpers::{error_from_shortlinker, success_response};
use super::types::{ReloadResponse, ValueType};

/// 配置项响应
#[derive(Debug, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
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
#[derive(Debug, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigUpdateRequest {
    pub value: String,
}

/// 配置更新响应
#[derive(Debug, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigUpdateResponse {
    pub key: String,
    pub value: String,
    pub requires_restart: bool,
    pub is_sensitive: bool,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub message: Option<String>,
}

/// 配置历史记录响应
#[derive(Debug, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigHistoryResponse {
    pub id: i32,
    pub config_key: String,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub old_value: Option<String>,
    pub new_value: String,
    pub changed_at: String,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub changed_by: Option<String>,
}

// ========== Config Action API types ==========

/// 配置 action 执行请求
#[derive(Debug, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigActionRequest {
    pub action: crate::config::types::ActionType,
}

/// 配置 action 执行响应
#[derive(Debug, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigActionResponse {
    pub value: String,
}

/// 执行并保存响应（安全版本，不返回密钥值）
#[derive(Debug, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ExecuteAndSaveResponse {
    pub success: bool,
    pub requires_restart: bool,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(
    all(debug_assertions, feature = "openapi"),
    derive(utoipa::IntoParams, utoipa::ToSchema)
)]
pub struct HistoryQuery {
    pub limit: Option<u64>,
}

// ========== Handlers ==========

/// 获取所有配置
#[aster_forge_api_docs_macros::path(
    get,
    path = "/admin/v1/config",
    tag = "config",
    operation_id = "list_configs",
    responses((status = 200, description = "Runtime configuration", body = super::types::ApiResponse<Vec<ConfigItemResponse>>)),
)]
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
#[aster_forge_api_docs_macros::path(
    get,
    path = "/admin/v1/config/{key}",
    tag = "config",
    operation_id = "get_config",
    params(("key" = String, Path, description = "Configuration key")),
    responses(
        (status = 200, description = "Configuration item", body = super::types::ApiResponse<ConfigItemResponse>),
        (status = 404, description = "Configuration key not found"),
    ),
)]
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
#[aster_forge_api_docs_macros::path(
    put,
    path = "/admin/v1/config/{key}",
    tag = "config",
    operation_id = "update_config",
    params(("key" = String, Path, description = "Configuration key")),
    request_body = ConfigUpdateRequest,
    responses(
        (status = 200, description = "Configuration updated", body = super::types::ApiResponse<ConfigUpdateResponse>),
        (status = 400, description = "Invalid configuration value"),
        (status = 404, description = "Configuration key not found"),
    ),
)]
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
#[aster_forge_api_docs_macros::path(
    get,
    path = "/admin/v1/config/{key}/history",
    tag = "config",
    operation_id = "get_config_history",
    params(
        ("key" = String, Path, description = "Configuration key"),
        HistoryQuery,
    ),
    responses((status = 200, description = "Configuration history", body = super::types::ApiResponse<Vec<ConfigHistoryResponse>>)),
)]
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
#[aster_forge_api_docs_macros::path(
    post,
    path = "/admin/v1/config/reload",
    tag = "config",
    operation_id = "reload_config",
    responses(
        (status = 200, description = "Configuration reloaded", body = super::types::ApiResponse<ReloadResponse>),
        (status = 500, description = "Configuration reload failed"),
    ),
)]
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
#[aster_forge_api_docs_macros::path(
    get,
    path = "/admin/v1/config/schema",
    tag = "config",
    operation_id = "get_config_schema",
    responses((status = 200, description = "Configuration schema", body = super::types::ApiResponse<Vec<crate::config::ConfigSchema>>)),
)]
pub async fn get_config_schema(_req: HttpRequest) -> ActixResult<impl Responder> {
    let schemas = crate::config::get_all_schemas().clone();
    Ok(success_response(schemas))
}

/// 执行配置 action（如生成 token）
///
/// POST /admin/v1/config/{key}/action
#[aster_forge_api_docs_macros::path(
    post,
    path = "/admin/v1/config/{key}/action",
    tag = "config",
    operation_id = "execute_config_action",
    params(("key" = String, Path, description = "Configuration key")),
    request_body = ConfigActionRequest,
    responses(
        (status = 200, description = "Action result", body = super::types::ApiResponse<ConfigActionResponse>),
        (status = 400, description = "Unsupported action"),
    ),
)]
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
#[aster_forge_api_docs_macros::path(
    post,
    path = "/admin/v1/config/{key}/execute-and-save",
    tag = "config",
    operation_id = "execute_and_save_config_action",
    params(("key" = String, Path, description = "Configuration key")),
    request_body = ConfigActionRequest,
    responses(
        (status = 200, description = "Action executed and value saved", body = super::types::ApiResponse<ExecuteAndSaveResponse>),
        (status = 400, description = "Unsupported action"),
    ),
)]
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
