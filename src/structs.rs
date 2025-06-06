pub use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    #[cfg(unix)]
    pub unix_socket_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ShortLink {
    pub code: String,
    pub target: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageSerializableShortLink {
    pub short_code: String,
    pub target_url: String,
    pub created_at: String,
    pub expires_at: Option<String>,

    #[serde(default)]
    pub click: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SerializableShortLink {
    pub short_code: String,
    pub target_url: String,
    pub created_at: String,
    pub expires_at: Option<String>,
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
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetLinksQuery {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub created_after: Option<String>,
    pub created_before: Option<String>,
    pub only_expired: Option<bool>,
    pub only_active: Option<bool>,
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

#[derive(Clone, Debug)]
pub struct AppStartTime {
    pub start_datetime: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct TimeParser;

#[derive(Debug, Clone)]
pub struct CliParser;

#[derive(Debug, Clone)]
pub struct ProcessManager;

#[derive(Debug, Clone)]
pub struct AdminService;

#[derive(Debug, Clone)]
pub struct RedirectService;
