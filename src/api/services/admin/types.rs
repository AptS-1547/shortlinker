//! Admin API 类型定义

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::storage::ShortLink;

/// 输出目录常量
pub const TS_EXPORT_PATH: &str = "../admin-panel/src/services/types.generated.ts";

/// 配置值类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    String,
    Int,
    Bool,
    Json,
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            Self::Json => write!(f, "json"),
        }
    }
}

impl std::str::FromStr for ValueType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(Self::String),
            "int" => Ok(Self::Int),
            "bool" => Ok(Self::Bool),
            "json" => Ok(Self::Json),
            _ => Err(format!("Unknown value type: {}", s)),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct LoginCredentials {
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub data: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct PostNewLink {
    pub code: Option<String>,
    pub target: String,
    pub expires_at: Option<String>,
    pub password: Option<String>,
    pub force: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
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
pub struct PaginatedResponse<T> {
    pub code: i32,
    pub data: T,
    pub pagination: PaginationInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct PaginationInfo {
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
}

// Batch operation request/response types
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct BatchCreateRequest {
    pub links: Vec<PostNewLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct BatchUpdateRequest {
    pub updates: Vec<BatchUpdateItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct BatchUpdateItem {
    pub code: String,
    pub payload: PostNewLink,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct BatchDeleteRequest {
    pub codes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct BatchResponse {
    pub success: Vec<String>,
    pub failed: Vec<BatchFailedItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct BatchFailedItem {
    pub code: String,
    pub error: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct LinkResponse {
    pub code: String,
    pub target: String,
    pub created_at: String,
    pub expires_at: Option<String>,
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
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct StatsResponse {
    pub total_links: usize,
    pub total_clicks: usize,
    pub active_links: usize,
}

// ============ 健康检查相关类型 ============

/// 存储后端信息
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct HealthStorageBackend {
    pub storage_type: String,
    pub support_click: bool,
}

/// 存储健康检查状态
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct HealthStorageCheck {
    pub status: String,
    pub links_count: Option<usize>,
    pub backend: HealthStorageBackend,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 健康检查项容器
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct HealthChecks {
    pub storage: HealthStorageCheck,
}

/// 健康检查响应
#[derive(Serialize, Deserialize, Clone, Debug, TS)]
#[ts(
    export,
    export_to = TS_EXPORT_PATH
)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub uptime: u32,
    pub checks: HealthChecks,
    pub response_time_ms: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::services::admin::config_ops::{
        ConfigHistoryResponse, ConfigItemResponse, ConfigUpdateRequest, ConfigUpdateResponse,
    };

    #[test]
    fn export_typescript_types() {
        // 运行此测试会自动生成 TypeScript 类型文件
        // cargo test export_typescript_types -- --nocapture

        // Admin types
        LoginCredentials::export_all().expect("Failed to export LoginCredentials");
        PostNewLink::export_all().expect("Failed to export PostNewLink");
        GetLinksQuery::export_all().expect("Failed to export GetLinksQuery");
        PaginationInfo::export_all().expect("Failed to export PaginationInfo");
        BatchCreateRequest::export_all().expect("Failed to export BatchCreateRequest");
        BatchUpdateRequest::export_all().expect("Failed to export BatchUpdateRequest");
        BatchUpdateItem::export_all().expect("Failed to export BatchUpdateItem");
        BatchDeleteRequest::export_all().expect("Failed to export BatchDeleteRequest");
        BatchResponse::export_all().expect("Failed to export BatchResponse");
        BatchFailedItem::export_all().expect("Failed to export BatchFailedItem");
        LinkResponse::export_all().expect("Failed to export LinkResponse");
        StatsResponse::export_all().expect("Failed to export StatsResponse");

        // Health check types
        HealthStorageBackend::export_all().expect("Failed to export HealthStorageBackend");
        HealthStorageCheck::export_all().expect("Failed to export HealthStorageCheck");
        HealthChecks::export_all().expect("Failed to export HealthChecks");
        HealthResponse::export_all().expect("Failed to export HealthResponse");

        // Config types
        ValueType::export_all().expect("Failed to export ValueType");
        ConfigItemResponse::export_all().expect("Failed to export ConfigItemResponse");
        ConfigUpdateRequest::export_all().expect("Failed to export ConfigUpdateRequest");
        ConfigUpdateResponse::export_all().expect("Failed to export ConfigUpdateResponse");
        ConfigHistoryResponse::export_all().expect("Failed to export ConfigHistoryResponse");

        println!("TypeScript types exported to {}", TS_EXPORT_PATH);
    }
}
