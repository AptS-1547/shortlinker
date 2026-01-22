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

/// IPC commands sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcCommand {
    /// Heartbeat check - confirm server is alive and responsive
    Ping,

    /// Reload request with specific target
    Reload {
        target: ReloadTarget,
    },

    /// Query current server status
    GetStatus,

    /// Request graceful shutdown
    Shutdown,
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
