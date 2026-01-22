//! IPC client
//!
//! Provides functions for CLI to communicate with the running server.

use bytes::BytesMut;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

use super::platform::{IpcPlatform, PlatformIpc};
use super::protocol::{decode, encode};
use super::types::{ImportLinkData, IpcCommand, IpcError, IpcResponse};
use crate::system::reload::ReloadTarget;

/// Default timeout for IPC operations
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Reload operation timeout (longer since reload can take time)
const RELOAD_TIMEOUT: Duration = Duration::from_secs(30);

/// Import/export operation timeout (can be long for large datasets)
const BULK_TIMEOUT: Duration = Duration::from_secs(60);

/// Check if the server is running
///
/// This performs a quick synchronous check by testing socket connectivity.
pub fn is_server_running() -> bool {
    PlatformIpc::is_server_running()
}

/// Send an IPC command and wait for response
///
/// Uses the default timeout of 5 seconds.
pub async fn send_command(cmd: IpcCommand) -> Result<IpcResponse, IpcError> {
    let timeout_duration = match &cmd {
        IpcCommand::Reload { .. } => RELOAD_TIMEOUT,
        IpcCommand::ImportLinks { .. } | IpcCommand::ExportLinks => BULK_TIMEOUT,
        _ => DEFAULT_TIMEOUT,
    };
    send_command_with_timeout(cmd, timeout_duration).await
}

/// Send an IPC command with a custom timeout
pub async fn send_command_with_timeout(
    cmd: IpcCommand,
    timeout_duration: Duration,
) -> Result<IpcResponse, IpcError> {
    // Connect to the server
    let mut stream = timeout(timeout_duration, PlatformIpc::connect())
        .await
        .map_err(|_| IpcError::Timeout)?
        .map_err(IpcError::from)?;

    // Encode and send the command
    let data = encode(&cmd).map_err(|e| IpcError::ProtocolError(e.to_string()))?;
    stream.write_all(&data).await.map_err(IpcError::IoError)?;
    stream.flush().await.map_err(IpcError::IoError)?;

    // Read the response
    let mut buf = BytesMut::with_capacity(4096);
    let mut read_buf = [0u8; 1024];

    loop {
        let n = timeout(timeout_duration, stream.read(&mut read_buf))
            .await
            .map_err(|_| IpcError::Timeout)?
            .map_err(IpcError::IoError)?;

        if n == 0 {
            return Err(IpcError::ProtocolError(
                "Connection closed before receiving response".to_string(),
            ));
        }

        buf.extend_from_slice(&read_buf[..n]);

        // Try to decode the response
        if let Some(response) =
            decode::<IpcResponse>(&mut buf).map_err(|e| IpcError::ProtocolError(e.to_string()))?
        {
            return Ok(response);
        }
    }
}

/// Send a ping command and return version and uptime
///
/// Returns `(version, uptime_secs)` on success.
pub async fn ping() -> Result<(String, u64), IpcError> {
    match send_command(IpcCommand::Ping).await? {
        IpcResponse::Pong {
            version,
            uptime_secs,
        } => Ok((version, uptime_secs)),
        IpcResponse::Error { code, message } => {
            Err(IpcError::ProtocolError(format!("{}: {}", code, message)))
        }
        other => Err(IpcError::ProtocolError(format!(
            "Unexpected response: {:?}",
            other
        ))),
    }
}

/// Send a reload command
///
/// Returns the IpcResponse which contains success/failure information.
pub async fn reload(target: ReloadTarget) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::Reload { target }).await
}

/// Get server status
pub async fn get_status() -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::GetStatus).await
}

/// Request server shutdown
pub async fn shutdown() -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::Shutdown).await
}

// ============ Link Management Client Functions ============

/// Add a new link via IPC
pub async fn add_link(
    code: Option<String>,
    target: String,
    force: bool,
    expires_at: Option<String>,
    password: Option<String>,
) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::AddLink {
        code,
        target,
        force,
        expires_at,
        password,
    })
    .await
}

/// Remove a link via IPC
pub async fn remove_link(code: String) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::RemoveLink { code }).await
}

/// Update a link via IPC
pub async fn update_link(
    code: String,
    target: String,
    expires_at: Option<String>,
    password: Option<String>,
) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::UpdateLink {
        code,
        target,
        expires_at,
        password,
    })
    .await
}

/// Get a single link via IPC
pub async fn get_link(code: String) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::GetLink { code }).await
}

/// List links via IPC
pub async fn list_links(
    page: u64,
    page_size: u64,
    search: Option<String>,
) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::ListLinks {
        page,
        page_size,
        search,
    })
    .await
}

/// Import links via IPC
pub async fn import_links(
    links: Vec<ImportLinkData>,
    overwrite: bool,
) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::ImportLinks { links, overwrite }).await
}

/// Export all links via IPC
pub async fn export_links() -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::ExportLinks).await
}

/// Get link statistics via IPC
pub async fn get_link_stats() -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::GetLinkStats).await
}
