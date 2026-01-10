//! Admin API 类型定义

use serde::{Deserialize, Serialize};

use crate::storage::ShortLink;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginCredentials {
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub data: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PostNewLink {
    pub code: Option<String>,
    pub target: String,
    pub expires_at: Option<String>,
    pub password: Option<String>,
    pub force: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PaginationInfo {
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
}

// Batch operation request/response types
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchCreateRequest {
    pub links: Vec<PostNewLink>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchUpdateRequest {
    pub updates: Vec<BatchUpdateItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchUpdateItem {
    pub code: String,
    pub payload: PostNewLink,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchDeleteRequest {
    pub codes: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchResponse {
    pub success: Vec<String>,
    pub failed: Vec<BatchFailedItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatchFailedItem {
    pub code: String,
    pub error: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
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
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StatsResponse {
    pub total_links: usize,
    pub total_clicks: usize,
    pub active_links: usize,
}
