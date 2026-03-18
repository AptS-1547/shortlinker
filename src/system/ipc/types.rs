//! IPC protocol type definitions
//!
//! Defines the types used for IPC communication:
//! - `IpcCommand`: Commands sent from client to server
//! - `IpcResponse`: Responses sent from server to client
//! - `IpcError`: IPC-related errors

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;

use crate::storage::ShortLink;
use crate::system::reload::ReloadTarget;

/// Import link data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportLinkData {
    pub code: String,
    pub target: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub password: Option<String>,
    pub click_count: usize,
}

impl From<&crate::services::ImportLinkItemRich> for ImportLinkData {
    fn from(l: &crate::services::ImportLinkItemRich) -> Self {
        Self {
            code: l.code.clone(),
            target: l.target.clone(),
            created_at: l.created_at.to_rfc3339(),
            expires_at: l.expires_at.map(|dt| dt.to_rfc3339()),
            password: l.password.clone(),
            click_count: l.click_count,
        }
    }
}

/// Import error data for IPC transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportErrorData {
    pub code: String,
    pub message: String,
    /// 结构化错误码（如 "E020"），用于客户端区分具体失败原因
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

/// Config item data for IPC transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigItemData {
    pub key: String,
    pub value: String,
    pub category: String,
    pub value_type: String,
    pub default_value: String,
    pub requires_restart: bool,
    pub editable: bool,
    pub sensitive: bool,
    pub description: String,
    pub enum_options: Option<Vec<String>>,
    pub updated_at: String,
}

/// Import progress phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImportPhase {
    Validating,
    ConflictCheck,
    Writing,
    CacheUpdate,
}

impl fmt::Display for ImportPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportPhase::Validating => write!(f, "Validating"),
            ImportPhase::ConflictCheck => write!(f, "Conflict check"),
            ImportPhase::Writing => write!(f, "Writing"),
            ImportPhase::CacheUpdate => write!(f, "Cache update"),
        }
    }
}

/// Config import item for batch import
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigImportItem {
    pub key: String,
    pub value: String,
}

/// IPC commands sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcCommand {
    /// Heartbeat check - confirm server is alive and responsive
    Ping,

    /// Reload request with specific target
    Reload { target: ReloadTarget },

    /// Query current server status
    GetStatus,

    /// Request graceful shutdown
    Shutdown,

    // ============ Link Management Commands ============
    /// Add a new short link
    AddLink {
        code: Option<String>,
        target: String,
        force: bool,
        expires_at: Option<String>,
        password: Option<String>,
    },

    /// Remove a short link
    RemoveLink { code: String },

    /// Batch delete short links
    BatchDeleteLinks { codes: Vec<String> },

    /// Update an existing short link
    UpdateLink {
        code: String,
        target: String,
        expires_at: Option<String>,
        password: Option<String>,
    },

    /// Get a single short link
    GetLink { code: String },

    /// List all short links with pagination
    ListLinks {
        page: u64,
        page_size: u64,
        search: Option<String>,
    },

    /// Import multiple links
    ImportLinks {
        links: Vec<ImportLinkData>,
        overwrite: bool,
        /// Request streaming progress reports (default false for backward compatibility)
        #[serde(default)]
        stream_progress: bool,
    },

    /// Export all links
    ExportLinks,

    /// Get link statistics
    GetLinkStats,

    // ============ Config Management Commands ============
    /// List all configurations
    ConfigList { category: Option<String> },

    /// Get a single configuration
    ConfigGet { key: String },

    /// Set a configuration value
    ConfigSet { key: String, value: String },

    /// Reset a configuration to default
    ConfigReset { key: String },

    /// Batch import configurations
    ConfigImport { configs: Vec<ConfigImportItem> },
}

/// IPC responses sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    /// Heartbeat response
    Pong {
        /// Server version
        version: String,
        /// Server uptime in seconds
        uptime_secs: u64,
    },

    /// Reload operation result
    ReloadResult {
        /// Whether the reload was successful
        success: bool,
        /// What was reloaded
        target: ReloadTarget,
        /// Duration in milliseconds
        duration_ms: u64,
        /// Optional message (error message on failure)
        message: Option<String>,
    },

    /// Current server status
    Status {
        /// Server version
        version: String,
        /// Server uptime in seconds
        uptime_secs: u64,
        /// Whether a reload is currently in progress
        is_reloading: bool,
        /// Last data reload time (ISO8601)
        last_data_reload: Option<String>,
        /// Last config reload time (ISO8601)
        last_config_reload: Option<String>,
        /// Number of links in cache
        links_count: usize,
    },

    /// Shutdown acknowledgment
    ShuttingDown,

    /// Error response
    Error {
        /// Error code
        code: String,
        /// Error message
        message: String,
    },

    // ============ Link Management Responses ============
    /// Link created successfully
    LinkCreated {
        link: ShortLink,
        /// Generated code if none was provided
        generated_code: bool,
    },

    /// Link deleted successfully
    LinkDeleted { code: String },

    /// Batch delete result
    BatchDeleteResult {
        deleted: Vec<String>,
        not_found: Vec<String>,
        errors: Vec<ImportErrorData>,
    },

    /// Link updated successfully
    LinkUpdated { link: ShortLink },

    /// Get link result
    LinkFound { link: Option<ShortLink> },

    /// List links result
    LinkList {
        links: Vec<ShortLink>,
        total: usize,
        page: u64,
        page_size: u64,
    },

    /// Import result
    ImportResult {
        success: usize,
        skipped: usize,
        failed: usize,
        errors: Vec<ImportErrorData>,
    },

    /// Import progress (streaming import)
    ImportProgress {
        phase: ImportPhase,
        processed: usize,
        total: usize,
        success: usize,
        skipped: usize,
        failed: usize,
    },

    /// Export result
    ExportResult { links: Vec<ShortLink> },

    /// Export chunk (streaming export)
    ExportChunk { links: Vec<ShortLink> },

    /// Export done marker (streaming export)
    ExportDone { total: usize },

    /// Stats result
    StatsResult {
        total_links: usize,
        total_clicks: i64,
        active_links: usize,
    },

    // ============ Config Management Responses ============
    /// Config list result
    ConfigListResult { configs: Vec<ConfigItemData> },

    /// Config get result
    ConfigGetResult { config: ConfigItemData },

    /// Config set result
    ConfigSetResult {
        key: String,
        value: String,
        requires_restart: bool,
        is_sensitive: bool,
        old_value: Option<String>,
        message: Option<String>,
    },

    /// Config reset result
    ConfigResetResult {
        key: String,
        value: String,
        requires_restart: bool,
        is_sensitive: bool,
        message: Option<String>,
    },

    /// Config import result
    ConfigImportResult {
        success: usize,
        skipped: usize,
        failed: usize,
        errors: Vec<ImportErrorData>,
    },
}

/// IPC connection errors
#[derive(Debug)]
pub enum IpcError {
    /// Server is not running (socket doesn't exist or cannot connect)
    ServerNotRunning,
    /// Connection timeout
    Timeout,
    /// Protocol error (invalid message format)
    ProtocolError(String),
    /// IO error during communication
    IoError(io::Error),
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IpcError::ServerNotRunning => write!(f, "Server is not running"),
            IpcError::Timeout => write!(f, "Connection timeout"),
            IpcError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
            IpcError::IoError(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for IpcError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            IpcError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for IpcError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::ConnectionRefused | io::ErrorKind::NotFound => {
                IpcError::ServerNotRunning
            }
            _ => IpcError::IoError(err),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_ipc_error_display() {
        assert_eq!(
            format!("{}", IpcError::ServerNotRunning),
            "Server is not running"
        );
        assert_eq!(format!("{}", IpcError::Timeout), "Connection timeout");
        assert_eq!(
            format!("{}", IpcError::ProtocolError("bad data".into())),
            "Protocol error: bad data"
        );

        let io_err = io::Error::other("test error");
        let ipc_err = IpcError::IoError(io_err);
        assert!(format!("{}", ipc_err).contains("IO error"));
    }

    #[test]
    fn test_ipc_error_from_io_connection_refused() {
        let io_err = io::Error::new(io::ErrorKind::ConnectionRefused, "refused");
        let ipc_err: IpcError = io_err.into();
        assert!(matches!(ipc_err, IpcError::ServerNotRunning));
    }

    #[test]
    fn test_ipc_error_from_io_not_found() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "not found");
        let ipc_err: IpcError = io_err.into();
        assert!(matches!(ipc_err, IpcError::ServerNotRunning));
    }

    #[test]
    fn test_ipc_error_from_io_other() {
        let io_err = io::Error::other("other error");
        let ipc_err: IpcError = io_err.into();
        assert!(matches!(ipc_err, IpcError::IoError(_)));
    }

    #[test]
    fn test_ipc_error_source() {
        let err = IpcError::ServerNotRunning;
        assert!(err.source().is_none());

        let io_err = io::Error::other("test");
        let err = IpcError::IoError(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_import_error_data_with_error_code() {
        let data = ImportErrorData {
            code: "dup".into(),
            message: "Link already exists".into(),
            error_code: Some("E021".into()),
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(json.contains("\"error_code\":\"E021\""));

        let decoded: ImportErrorData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.error_code, Some("E021".into()));
    }

    #[test]
    fn test_import_error_data_without_error_code() {
        let data = ImportErrorData {
            code: "key".into(),
            message: "unknown".into(),
            error_code: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(!json.contains("error_code"));

        let json_no_code = r#"{"code":"key","message":"unknown"}"#;
        let decoded: ImportErrorData = serde_json::from_str(json_no_code).unwrap();
        assert_eq!(decoded.error_code, None);
    }
}
