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
    pub expires_at: Option<String>,
    pub password: Option<String>,
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
    },

    /// Export all links
    ExportLinks,

    /// Get link statistics
    GetLinkStats,
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
        errors: Vec<String>,
    },

    /// Export result
    ExportResult { links: Vec<ShortLinkData> },

    /// Stats result
    StatsResult {
        total_links: usize,
        total_clicks: i64,
        active_links: usize,
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
            expires_at: None,
            password: Some("secret".into()),
        };
        let json = serde_json::to_string(&data).unwrap();
        let decoded: ImportLinkData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.code, "test");
        assert_eq!(decoded.password, Some("secret".into()));
    }
}
