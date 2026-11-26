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
    /// 配置 key，与 `config/definitions.rs` 注册的标识一致
    pub key: String,
    /// 展示用的当前值；敏感配置返回 `[REDACTED]` 占位符而非真实值
    pub value: String,
    /// 值类型
    pub value_type: ValueType,
    /// 修改后是否需要重启进程才能生效；否则热重载即时生效
    pub requires_restart: bool,
    /// 是否为敏感配置；该值为 `true` 时 `value` 只能是占位符，不能用于回填
    pub is_sensitive: bool,
    /// 最近一次更新时间，RFC 3339 格式
    pub updated_at: String,
}

/// 配置更新请求
#[derive(Debug, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigUpdateRequest {
    /// 要写入的新配置值，服务端按该 key 的 value_type 做校验
    pub value: String,
}

/// 配置更新响应
#[derive(Debug, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigUpdateResponse {
    /// 被更新的配置 key
    pub key: String,
    /// 更新后的展示值；敏感配置为 `[REDACTED]` 占位符
    pub value: String,
    /// 是否需要重启才能生效
    pub requires_restart: bool,
    /// 是否为敏感配置
    pub is_sensitive: bool,
    /// 更新触发的附加信息（如校验提示）；无附加信息时为 `null`
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub message: Option<String>,
}

/// 配置历史记录响应
#[derive(Debug, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigHistoryResponse {
    /// 历史记录主键，按时间递增
    pub id: i32,
    /// 发生变更的配置 key
    pub config_key: String,
    /// 变更前的值（敏感配置为 `[REDACTED]`）；首次赋值时为 `null`
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub old_value: Option<String>,
    /// 变更后的值（敏感配置为 `[REDACTED]`）
    pub new_value: String,
    /// 变更时间，RFC 3339 格式
    pub changed_at: String,
    /// 变更操作者标识；无法归因（如导入路径）时为 `null`
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub changed_by: Option<String>,
}

// ========== Config Action API types ==========

/// 配置 action 执行请求
#[derive(Debug, Deserialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigActionRequest {
    /// 要执行的 action 类型，只有注册了对应 action 的配置 key 才接受
    pub action: crate::config::types::ActionType,
}

/// 配置 action 执行响应
#[derive(Debug, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ConfigActionResponse {
    /// action 产出的值（如生成的 token）；敏感值由此接口直接返回，仅供调用方立即使用
    pub value: String,
}

/// 执行并保存响应（安全版本，不返回密钥值）
#[derive(Debug, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ExecuteAndSaveResponse {
    /// action 是否执行并保存成功
    pub success: bool,
    /// 新值是否需要重启才能生效
    pub requires_restart: bool,
    /// 执行结果的附加信息；无附加信息时为 `null`
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub message: Option<String>,
}

/// 配置历史查询参数
#[derive(Debug, Deserialize)]
#[cfg_attr(
    all(debug_assertions, feature = "openapi"),
    derive(utoipa::IntoParams, utoipa::ToSchema)
)]
pub struct HistoryQuery {
    /// 最多返回的历史记录条数，默认 20，服务端会截断到最大 100
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
