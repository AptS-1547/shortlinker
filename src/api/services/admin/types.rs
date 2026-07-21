//! Admin API 类型定义

use serde::{Deserialize, Serialize};

use crate::storage::ShortLink;

// Re-export ValueType from config module
pub use crate::config::ValueType;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LoginCredentials {
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PostNewLink {
    pub code: Option<String>,
    pub target: String,
    pub expires_at: Option<String>,
    pub password: Option<String>,
    pub force: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(
    all(debug_assertions, feature = "openapi"),
    derive(utoipa::IntoParams, utoipa::ToSchema)
)]
pub struct GetLinksQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub only_expired: Option<bool>,
    pub only_active: Option<bool>,
    pub search: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PaginatedResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub pagination: PaginationInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PaginationInfo {
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
}

// Batch operation request/response types
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchCreateRequest {
    pub links: Vec<PostNewLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchUpdateRequest {
    pub updates: Vec<BatchUpdateItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchUpdateItem {
    pub code: String,
    pub payload: PostNewLink,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchDeleteRequest {
    pub codes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchResponse {
    pub success: Vec<String>,
    pub failed: Vec<BatchFailedItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchFailedItem {
    pub code: String,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LinkResponse {
    pub code: String,
    pub target: String,
    pub created_at: String,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub expires_at: Option<String>,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub password: Option<String>,
    pub click_count: usize,
}

impl From<ShortLink> for LinkResponse {
    fn from(link: ShortLink) -> Self {
        Self {
            code: link.code,
            target: link.target,
            created_at: link.created_at.to_rfc3339(),
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            password: link.password,
            click_count: link.click,
        }
    }
}

/// 统计信息响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct StatsResponse {
    pub total_links: usize,
    pub total_clicks: usize,
    pub active_links: usize,
}

/// 简单消息响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct MessageResponse {
    pub message: String,
}

/// 认证成功响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AuthSuccessResponse {
    pub message: String,
    pub expires_in: u64,
}

/// Reload 成功响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ReloadResponse {
    pub message: String,
    pub duration_ms: u64,
}

// ============ 健康检查相关类型 ============

/// 存储后端信息
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthStorageBackend {
    pub storage_type: String,
    pub support_click: bool,
}

/// 存储健康检查状态
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthStorageCheck {
    pub status: String,
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub links_count: Option<usize>,
    pub backend: HealthStorageBackend,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 健康检查项容器
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthChecks {
    pub storage: HealthStorageCheck,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<HealthCacheCheck>,
}

/// 缓存健康检查状态
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthCacheCheck {
    pub status: String,
    pub cache_type: String,
    pub bloom_filter_enabled: bool,
    pub negative_cache_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 健康检查响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub uptime: u32,
    pub checks: HealthChecks,
    pub response_time_ms: u32,
}

// ============ 导出导入相关类型 ============

/// 导出查询参数
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[cfg_attr(
    all(debug_assertions, feature = "openapi"),
    derive(utoipa::IntoParams, utoipa::ToSchema)
)]
pub struct ExportQuery {
    pub search: Option<String>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub only_expired: Option<bool>,
    pub only_active: Option<bool>,
}

/// 导入模式 - 从 service 层 re-export
pub use crate::services::ImportMode;

/// 导入失败项
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ImportFailedItem {
    /// CSV 行号（1-based），None 表示行号未知（如 service 层返回的冲突项无法反查行号）
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub row: Option<usize>,
    pub code: String,
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i32>,
}

/// 导入响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ImportResponse {
    pub total_rows: usize,
    pub success_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub failed_items: Vec<ImportFailedItem>,
}

// Re-export CSV row types from shared csv_handler module
pub use crate::utils::csv_handler::{ClickLogCsvRow, CsvLinkRow};
