//! IPC protocol type definitions
//!
//! Defines the types used for IPC communication:
//! - `IpcCommand`: Commands sent from client to server
//! - `IpcResponse`: Responses sent from server to client
//! - `IpcError`: IPC-related errors

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io;

use crate::system::reload::ReloadTarget;

/// Short link data for IPC transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLinkData {
    pub code: String,
    pub target: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub password: Option<String>,
    pub click: i64,
}

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
        link: ShortLinkData,
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
    LinkUpdated { link: ShortLinkData },

    /// Get link result
    LinkFound { link: Option<ShortLinkData> },

    /// List links result
    LinkList {
        links: Vec<ShortLinkData>,
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
    ExportResult { links: Vec<ShortLinkData> },

    /// Export chunk (streaming export)
    ExportChunk { links: Vec<ShortLinkData> },

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
        // Map specific error kinds to more specific IPC errors
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
        // ServerNotRunning has no source
        let err = IpcError::ServerNotRunning;
        assert!(err.source().is_none());

        // IoError has source
        let io_err = io::Error::other("test");
        let err = IpcError::IoError(io_err);
        assert!(err.source().is_some());
    }

    #[test]
    fn test_ipc_command_serialization() {
        // Test Ping
        let cmd = IpcCommand::Ping;
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: IpcCommand = serde_json::from_str(&json).unwrap();
        assert!(matches!(decoded, IpcCommand::Ping));

        // Test AddLink
        let cmd = IpcCommand::AddLink {
            code: Some("test".into()),
            target: "https://example.com".into(),
            force: true,
            expires_at: Some("2025-12-31T23:59:59Z".into()),
            password: None,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: IpcCommand = serde_json::from_str(&json).unwrap();
        if let IpcCommand::AddLink {
            code,
            target,
            force,
            ..
        } = decoded
        {
            assert_eq!(code, Some("test".into()));
            assert_eq!(target, "https://example.com");
            assert!(force);
        } else {
            panic!("Expected AddLink");
        }
    }

    #[test]
    fn test_ipc_response_serialization() {
        // Test Pong
        let resp = IpcResponse::Pong {
            version: "1.0.0".into(),
            uptime_secs: 3600,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: IpcResponse = serde_json::from_str(&json).unwrap();
        if let IpcResponse::Pong {
            version,
            uptime_secs,
        } = decoded
        {
            assert_eq!(version, "1.0.0");
            assert_eq!(uptime_secs, 3600);
        } else {
            panic!("Expected Pong");
        }

        // Test Error
        let resp = IpcResponse::Error {
            code: "E001".into(),
            message: "Something went wrong".into(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: IpcResponse = serde_json::from_str(&json).unwrap();
        if let IpcResponse::Error { code, message } = decoded {
            assert_eq!(code, "E001");
            assert_eq!(message, "Something went wrong");
        } else {
            panic!("Expected Error");
        }
    }

    #[test]
    fn test_short_link_data_serialization() {
        let data = ShortLinkData {
            code: "abc123".into(),
            target: "https://example.com".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            expires_at: Some("2025-12-31T23:59:59Z".into()),
            password: None,
            click: 42,
        };
        let json = serde_json::to_string(&data).unwrap();
        let decoded: ShortLinkData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.code, "abc123");
        assert_eq!(decoded.click, 42);
    }

    #[test]
    fn test_import_link_data_serialization() {
        let data = ImportLinkData {
            code: "test".into(),
            target: "https://example.com".into(),
            created_at: "2025-01-01T00:00:00Z".into(),
            expires_at: None,
            password: Some("secret".into()),
            click_count: 10,
        };
        let json = serde_json::to_string(&data).unwrap();
        let decoded: ImportLinkData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.code, "test");
        assert_eq!(decoded.password, Some("secret".into()));
        assert_eq!(decoded.created_at, "2025-01-01T00:00:00Z");
        assert_eq!(decoded.click_count, 10);
    }

    #[test]
    fn test_config_item_data_serialization() {
        let data = ConfigItemData {
            key: "auth.admin_token".into(),
            value: "[REDACTED]".into(),
            category: "auth".into(),
            value_type: "string".into(),
            default_value: "".into(),
            requires_restart: false,
            editable: true,
            sensitive: true,
            description: "Admin token".into(),
            enum_options: None,
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let decoded: ConfigItemData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.key, "auth.admin_token");
        assert!(decoded.sensitive);
        assert!(decoded.enum_options.is_none());
    }

    #[test]
    fn test_config_import_item_serialization() {
        let item = ConfigImportItem {
            key: "features.random_code_length".into(),
            value: "8".into(),
        };
        let json = serde_json::to_string(&item).unwrap();
        let decoded: ConfigImportItem = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.key, "features.random_code_length");
        assert_eq!(decoded.value, "8");
    }

    #[test]
    fn test_config_command_serialization() {
        // ConfigList
        let cmd = IpcCommand::ConfigList {
            category: Some("auth".into()),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: IpcCommand = serde_json::from_str(&json).unwrap();
        if let IpcCommand::ConfigList { category } = decoded {
            assert_eq!(category, Some("auth".into()));
        } else {
            panic!("Expected ConfigList");
        }

        // ConfigSet
        let cmd = IpcCommand::ConfigSet {
            key: "features.random_code_length".into(),
            value: "8".into(),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: IpcCommand = serde_json::from_str(&json).unwrap();
        if let IpcCommand::ConfigSet { key, value } = decoded {
            assert_eq!(key, "features.random_code_length");
            assert_eq!(value, "8");
        } else {
            panic!("Expected ConfigSet");
        }

        // ConfigImport
        let cmd = IpcCommand::ConfigImport {
            configs: vec![ConfigImportItem {
                key: "k".into(),
                value: "v".into(),
            }],
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: IpcCommand = serde_json::from_str(&json).unwrap();
        if let IpcCommand::ConfigImport { configs } = decoded {
            assert_eq!(configs.len(), 1);
        } else {
            panic!("Expected ConfigImport");
        }
    }

    #[test]
    fn test_config_response_serialization() {
        // ConfigSetResult
        let resp = IpcResponse::ConfigSetResult {
            key: "k".into(),
            value: "v".into(),
            requires_restart: false,
            is_sensitive: false,
            old_value: Some("old".into()),
            message: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: IpcResponse = serde_json::from_str(&json).unwrap();
        if let IpcResponse::ConfigSetResult { key, old_value, .. } = decoded {
            assert_eq!(key, "k");
            assert_eq!(old_value, Some("old".into()));
        } else {
            panic!("Expected ConfigSetResult");
        }

        // ConfigImportResult
        let resp = IpcResponse::ConfigImportResult {
            success: 5,
            skipped: 1,
            failed: 0,
            errors: vec![],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: IpcResponse = serde_json::from_str(&json).unwrap();
        if let IpcResponse::ConfigImportResult {
            success, skipped, ..
        } = decoded
        {
            assert_eq!(success, 5);
            assert_eq!(skipped, 1);
        } else {
            panic!("Expected ConfigImportResult");
        }

        // ConfigImportResult with errors
        let resp = IpcResponse::ConfigImportResult {
            success: 1,
            skipped: 0,
            failed: 1,
            errors: vec![ImportErrorData {
                code: "auth.token".into(),
                message: "read-only".into(),
                error_code: None,
            }],
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: IpcResponse = serde_json::from_str(&json).unwrap();
        if let IpcResponse::ConfigImportResult { errors, .. } = decoded {
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].code, "auth.token");
            assert_eq!(errors[0].message, "read-only");
        } else {
            panic!("Expected ConfigImportResult");
        }
    }

    #[test]
    fn test_import_error_data_with_error_code() {
        // error_code 存在时序列化
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
        // error_code 为 None 时不出现在 JSON 中（skip_serializing_if）
        let data = ImportErrorData {
            code: "key".into(),
            message: "unknown".into(),
            error_code: None,
        };
        let json = serde_json::to_string(&data).unwrap();
        assert!(!json.contains("error_code"));

        // 反序列化时缺失字段也能成功（serde default）
        let json_no_code = r#"{"code":"key","message":"unknown"}"#;
        let decoded: ImportErrorData = serde_json::from_str(json_no_code).unwrap();
        assert_eq!(decoded.error_code, None);
    }
}
