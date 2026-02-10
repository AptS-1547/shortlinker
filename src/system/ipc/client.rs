//! IPC client
//!
//! Provides functions for CLI to communicate with the running server.

use bytes::BytesMut;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::timeout;

use super::platform::{IpcPlatform, PlatformIpc};
use super::protocol::{decode, encode};
use super::types::{
    ConfigImportItem, ImportLinkData, IpcCommand, IpcError, IpcResponse, ShortLinkData,
};
use crate::system::reload::ReloadTarget;

/// Check if the server is running
///
/// This performs a quick synchronous check by testing socket connectivity.
pub fn is_server_running() -> bool {
    PlatformIpc::is_server_running()
}

/// Send an IPC command and wait for response
///
/// Uses timeout from configuration based on command type.
pub async fn send_command(cmd: IpcCommand) -> Result<IpcResponse, IpcError> {
    let config = crate::config::get_config();
    let timeout_duration = match &cmd {
        IpcCommand::Reload { .. } => config.ipc.reload_timeout_duration(),
        IpcCommand::ImportLinks { .. } | IpcCommand::ExportLinks => {
            config.ipc.bulk_timeout_duration()
        }
        _ => config.ipc.default_timeout(),
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

/// Batch delete links via IPC
pub async fn batch_delete_links(codes: Vec<String>) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::BatchDeleteLinks { codes }).await
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
    send_command(IpcCommand::ImportLinks {
        links,
        overwrite,
        stream_progress: false,
    })
    .await
}

/// Import links via IPC with streaming progress reports.
///
/// Sends ImportLinks with stream_progress=true and receives streaming
/// ImportProgress + ImportResult responses. Calls `on_progress` for each
/// ImportProgress message. Returns the final ImportResult response.
pub async fn import_links_streaming(
    links: Vec<ImportLinkData>,
    overwrite: bool,
    on_progress: impl Fn(&super::types::ImportPhase, usize, usize),
) -> Result<IpcResponse, IpcError> {
    let config = crate::config::get_config();
    let timeout_duration = config.ipc.bulk_timeout_duration();

    // Connect to the server
    let mut stream = timeout(timeout_duration, PlatformIpc::connect())
        .await
        .map_err(|_| IpcError::Timeout)?
        .map_err(IpcError::from)?;

    // Encode and send the command
    let cmd = IpcCommand::ImportLinks {
        links,
        overwrite,
        stream_progress: true,
    };
    let data = encode(&cmd).map_err(|e| IpcError::ProtocolError(e.to_string()))?;
    stream.write_all(&data).await.map_err(IpcError::IoError)?;
    stream.flush().await.map_err(IpcError::IoError)?;

    // Read streaming responses
    let mut buf = BytesMut::with_capacity(4096);
    let mut read_buf = [0u8; 4096];

    loop {
        // Try to decode any buffered responses first
        loop {
            match decode::<IpcResponse>(&mut buf)
                .map_err(|e| IpcError::ProtocolError(e.to_string()))?
            {
                Some(IpcResponse::ImportProgress {
                    phase,
                    processed,
                    total,
                    ..
                }) => {
                    on_progress(&phase, processed, total);
                    continue;
                }
                Some(resp @ IpcResponse::ImportResult { .. }) => {
                    return Ok(resp);
                }
                Some(IpcResponse::Error { code, message }) => {
                    return Err(IpcError::ProtocolError(format!("{}: {}", code, message)));
                }
                Some(other) => {
                    return Err(IpcError::ProtocolError(format!(
                        "Unexpected response during streaming import: {:?}",
                        other
                    )));
                }
                None => {
                    break; // Need more data
                }
            }
        }

        // Read more data from socket
        let n = timeout(timeout_duration, stream.read(&mut read_buf))
            .await
            .map_err(|_| IpcError::Timeout)?
            .map_err(IpcError::IoError)?;

        if n == 0 {
            return Err(IpcError::ProtocolError(
                "Connection closed during streaming import".to_string(),
            ));
        }

        buf.extend_from_slice(&read_buf[..n]);
    }
}

/// Export all links via IPC (streaming)
///
/// Sends ExportLinks command and receives streaming ExportChunk + ExportDone responses.
/// Returns all exported links collected from chunks.
pub async fn export_links() -> Result<Vec<ShortLinkData>, IpcError> {
    let config = crate::config::get_config();
    let timeout_duration = config.ipc.bulk_timeout_duration();

    // Connect to the server
    let mut stream = timeout(timeout_duration, PlatformIpc::connect())
        .await
        .map_err(|_| IpcError::Timeout)?
        .map_err(IpcError::from)?;

    // Encode and send the command
    let data =
        encode(&IpcCommand::ExportLinks).map_err(|e| IpcError::ProtocolError(e.to_string()))?;
    stream.write_all(&data).await.map_err(IpcError::IoError)?;
    stream.flush().await.map_err(IpcError::IoError)?;

    // Read streaming responses
    let mut buf = BytesMut::with_capacity(4096);
    let mut read_buf = [0u8; 4096];
    let mut all_links: Vec<ShortLinkData> = Vec::new();

    loop {
        // Try to decode any buffered responses first
        loop {
            match decode::<IpcResponse>(&mut buf)
                .map_err(|e| IpcError::ProtocolError(e.to_string()))?
            {
                Some(IpcResponse::ExportChunk { links }) => {
                    all_links.extend(links);
                    continue; // Try decoding more from buffer
                }
                Some(IpcResponse::ExportDone { .. }) => {
                    return Ok(all_links);
                }
                Some(IpcResponse::Error { code, message }) => {
                    return Err(IpcError::ProtocolError(format!("{}: {}", code, message)));
                }
                Some(other) => {
                    return Err(IpcError::ProtocolError(format!(
                        "Unexpected response during streaming export: {:?}",
                        other
                    )));
                }
                None => {
                    break; // Need more data
                }
            }
        }

        // Read more data from socket
        let n = timeout(timeout_duration, stream.read(&mut read_buf))
            .await
            .map_err(|_| IpcError::Timeout)?
            .map_err(IpcError::IoError)?;

        if n == 0 {
            return Err(IpcError::ProtocolError(
                "Connection closed during streaming export".to_string(),
            ));
        }

        buf.extend_from_slice(&read_buf[..n]);
    }
}

/// Get link statistics via IPC
pub async fn get_link_stats() -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::GetLinkStats).await
}

// ============ Config Management Client Functions ============

/// List all configurations via IPC
pub async fn config_list(category: Option<String>) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::ConfigList { category }).await
}

/// Get a single configuration via IPC
pub async fn config_get(key: String) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::ConfigGet { key }).await
}

/// Set a configuration value via IPC
pub async fn config_set(key: String, value: String) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::ConfigSet { key, value }).await
}

/// Reset a configuration to default via IPC
pub async fn config_reset(key: String) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::ConfigReset { key }).await
}

/// Batch import configurations via IPC
pub async fn config_import(configs: Vec<ConfigImportItem>) -> Result<IpcResponse, IpcError> {
    send_command(IpcCommand::ConfigImport { configs }).await
}
