//! System operations client (IPC-only, no fallback)
//!
//! Server status, reload, and shutdown require a running server.

use crate::system::ipc::{self, IpcCommand, IpcResponse};
use crate::system::reload::ReloadTarget;

use super::ClientError;

/// Server status information
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub version: String,
    pub uptime_secs: u64,
    pub is_reloading: bool,
    pub last_data_reload: Option<String>,
    pub last_config_reload: Option<String>,
    pub links_count: usize,
}

/// Reload operation result
#[derive(Debug, Clone)]
pub struct ReloadResult {
    pub success: bool,
    pub target: ReloadTarget,
    pub duration_ms: u64,
    pub message: Option<String>,
}

/// System operations client — IPC-only, no fallback.
///
/// These operations require a running server.
pub struct SystemClient;

impl Default for SystemClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemClient {
    pub fn new() -> Self {
        Self
    }

    /// Ping the server, returns (version, uptime_secs)
    pub async fn ping(&self) -> Result<(String, u64), ClientError> {
        ipc::ping().await.map_err(ClientError::Ipc)
    }

    /// Get server status
    pub async fn get_status(&self) -> Result<ServerStatus, ClientError> {
        match ipc::send_command(IpcCommand::GetStatus)
            .await
            .map_err(ClientError::Ipc)?
        {
            IpcResponse::Status {
                version,
                uptime_secs,
                is_reloading,
                last_data_reload,
                last_config_reload,
                links_count,
            } => Ok(ServerStatus {
                version,
                uptime_secs,
                is_reloading,
                last_data_reload,
                last_config_reload,
                links_count,
            }),
            IpcResponse::Error { code, message } => Err(ClientError::ServerError { code, message }),
            other => Err(ClientError::Ipc(
                crate::system::ipc::IpcError::ProtocolError(format!(
                    "Unexpected response: {:?}",
                    other
                )),
            )),
        }
    }

    /// Request config/data reload
    pub async fn reload(&self, target: ReloadTarget) -> Result<ReloadResult, ClientError> {
        match ipc::reload(target).await.map_err(ClientError::Ipc)? {
            IpcResponse::ReloadResult {
                success,
                target,
                duration_ms,
                message,
            } => Ok(ReloadResult {
                success,
                target,
                duration_ms,
                message,
            }),
            IpcResponse::Error { code, message } => Err(ClientError::ServerError { code, message }),
            other => Err(ClientError::Ipc(
                crate::system::ipc::IpcError::ProtocolError(format!(
                    "Unexpected response: {:?}",
                    other
                )),
            )),
        }
    }

    /// Request graceful shutdown
    pub async fn shutdown(&self) -> Result<(), ClientError> {
        match ipc::send_command(IpcCommand::Shutdown)
            .await
            .map_err(ClientError::Ipc)?
        {
            IpcResponse::ShuttingDown => Ok(()),
            IpcResponse::Error { code, message } => Err(ClientError::ServerError { code, message }),
            other => Err(ClientError::Ipc(
                crate::system::ipc::IpcError::ProtocolError(format!(
                    "Unexpected response: {:?}",
                    other
                )),
            )),
        }
    }
}
