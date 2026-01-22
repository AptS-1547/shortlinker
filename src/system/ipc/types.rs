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

/// Socket path for Unix Domain Socket
pub const SOCKET_PATH_UNIX: &str = "./shortlinker.sock";

/// Named pipe path for Windows
pub const PIPE_NAME_WINDOWS: &str = r"\\.\pipe\shortlinker";

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
