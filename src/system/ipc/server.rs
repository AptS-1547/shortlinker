//! IPC server
//!
//! Runs alongside the HTTP server to handle IPC commands from CLI.

use bytes::BytesMut;
use futures_util::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info, warn};

use super::handler::{handle_command, to_link_data};
use super::platform::{IpcPlatform, PlatformIpc};
use super::protocol::{decode, encode};
use super::types::{IpcCommand, IpcResponse};

/// Start the IPC server
///
/// This spawns a background task that listens for IPC connections
/// and handles commands. Returns a JoinHandle that can be used to
/// wait for or cancel the server.
///
/// # Panics
///
/// Does not panic, but logs errors if the server cannot start.
pub async fn start_ipc_server() -> Option<tokio::task::JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        let mut listener = match PlatformIpc::bind().await {
            Ok(l) => {
                info!("IPC server listening on {}", PlatformIpc::socket_path());
                l
            }
            Err(e) => {
                error!("Failed to start IPC server: {}", e);
                return;
            }
        };

        loop {
            match PlatformIpc::accept(&mut listener).await {
                Ok(stream) => {
                    // Handle each connection in a separate task
                    tokio::spawn(handle_connection(stream));
                }
                Err(e) => {
                    warn!("Failed to accept IPC connection: {}", e);
                }
            }
        }
    });

    Some(handle)
}

/// Send a single IpcResponse over the stream
async fn send_response<S>(stream: &mut S, response: &IpcResponse) -> Result<(), ()>
where
    S: tokio::io::AsyncWrite + Unpin,
{
    match encode(response) {
        Ok(data) => {
            if let Err(e) = stream.write_all(&data).await {
                error!("Failed to send IPC response: {}", e);
                return Err(());
            }
            if let Err(e) = stream.flush().await {
                error!("Failed to flush IPC response: {}", e);
                return Err(());
            }
            Ok(())
        }
        Err(e) => {
            error!("Failed to encode IPC response: {}", e);
            let error_response = IpcResponse::Error {
                code: "ENCODE_ERROR".to_string(),
                message: e.to_string(),
            };
            if let Ok(data) = encode(&error_response) {
                let _ = stream.write_all(&data).await;
                let _ = stream.flush().await;
            }
            Err(())
        }
    }
}

/// Handle streaming export: sends ExportChunk packets followed by ExportDone
async fn handle_streaming_export<S>(stream: &mut S) -> Result<(), ()>
where
    S: tokio::io::AsyncWrite + Unpin,
{
    let Some(mut link_stream) = super::handler::export_links_stream() else {
        let err = IpcResponse::Error {
            code: "SERVICE_UNAVAILABLE".to_string(),
            message: "Service not initialized".to_string(),
        };
        return send_response(stream, &err).await;
    };

    let mut total = 0usize;

    while let Some(batch_result) = link_stream.next().await {
        match batch_result {
            Ok(links) => {
                let count = links.len();
                if count == 0 {
                    continue;
                }
                total += count;
                let chunk = IpcResponse::ExportChunk {
                    links: links.iter().map(to_link_data).collect(),
                };
                send_response(stream, &chunk).await?;
                debug!(
                    "IPC export: sent chunk of {} links (total: {})",
                    count, total
                );
            }
            Err(e) => {
                error!("IPC export stream error: {}", e);
                let err = IpcResponse::Error {
                    code: "EXPORT_STREAM_ERROR".to_string(),
                    message: e.to_string(),
                };
                let _ = send_response(stream, &err).await;
                return Err(());
            }
        }
    }

    let done = IpcResponse::ExportDone { total };
    send_response(stream, &done).await?;
    info!("IPC export completed: {} total links", total);
    Ok(())
}

/// Handle a single IPC connection
async fn handle_connection<S>(mut stream: S)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    let mut buf = BytesMut::with_capacity(4096);
    let mut read_buf = [0u8; 1024];

    debug!("New IPC connection established");

    loop {
        // Read data from the stream
        match stream.read(&mut read_buf).await {
            Ok(0) => {
                debug!("IPC client disconnected");
                break;
            }
            Ok(n) => {
                buf.extend_from_slice(&read_buf[..n]);
            }
            Err(e) => {
                debug!("IPC read error: {}", e);
                break;
            }
        }

        // Try to decode and process commands
        loop {
            match decode::<IpcCommand>(&mut buf) {
                Ok(Some(cmd)) => {
                    debug!("Received IPC command: {:?}", cmd);

                    match cmd {
                        IpcCommand::ExportLinks => {
                            // Streaming export: send multiple responses
                            if handle_streaming_export(&mut stream).await.is_err() {
                                return;
                            }
                        }
                        other_cmd => {
                            // Single response commands
                            let response = handle_command(other_cmd).await;
                            if send_response(&mut stream, &response).await.is_err() {
                                return;
                            }
                        }
                    }
                }
                Ok(None) => {
                    // Need more data
                    break;
                }
                Err(e) => {
                    error!("IPC protocol error: {}", e);
                    // Send error response and close connection
                    let error_response = IpcResponse::Error {
                        code: "PROTOCOL_ERROR".to_string(),
                        message: e.to_string(),
                    };
                    let _ = send_response(&mut stream, &error_response).await;
                    return;
                }
            }
        }
    }
}

/// Stop the IPC server and clean up
///
/// This removes the socket file (on Unix) to allow a clean restart.
pub fn stop_ipc_server() {
    PlatformIpc::cleanup();
    info!("IPC server stopped, socket cleaned up");
}
