//! IPC client
//!
//! Provides functions for CLI to communicate with the running server.

use bytes::BytesMut;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

use super::platform::{IpcPlatform, PlatformIpc};
use super::protocol::{decode, encode};
use super::types::{IpcCommand, IpcError, IpcResponse};
use crate::system::reload::ReloadTarget;

/// Default timeout for IPC operations
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Reload operation timeout (longer since reload can take time)
const RELOAD_TIMEOUT: Duration = Duration::from_secs(30);

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
