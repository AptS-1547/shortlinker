//! IPC server
//!
//! Runs alongside the HTTP server to handle IPC commands from CLI.

use bytes::BytesMut;
use futures_util::StreamExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

use super::handler::handle_command;
use super::platform::{IpcPlatform, PlatformIpc};
use super::protocol::{decode, encode};
use super::types::{IpcCommand, IpcResponse};

pub async fn run_ipc_server(shutdown_token: CancellationToken) {
    let mut listener = match PlatformIpc::bind().await {
        Ok(listener) => {
            info!("IPC server listening on {}", PlatformIpc::socket_path());
            listener
        }
        Err(error) => {
            error!(%error, "Failed to start IPC server");
            return;
        }
    };
    let mut connections = JoinSet::new();

    loop {
        tokio::select! {
            _ = shutdown_token.cancelled() => break,
            result = PlatformIpc::accept(&mut listener) => match result {
                Ok(stream) => {
                    connections.spawn(handle_connection(stream));
                }
                Err(error) => warn!(%error, "Failed to accept IPC connection"),
            },
        }
    }

    connections.abort_all();
    while connections.join_next().await.is_some() {}
    PlatformIpc::cleanup();
    info!("IPC server stopped, socket cleaned up");
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
                let chunk = IpcResponse::ExportChunk { links };
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

/// Handle streaming import: sends ImportProgress packets followed by ImportResult
async fn handle_streaming_import<S>(
    stream: &mut S,
    links: Vec<super::types::ImportLinkData>,
    overwrite: bool,
) -> Result<(), ()>
where
    S: tokio::io::AsyncWrite + Unpin,
{
    let Some(mut rx) = super::handler::import_links_with_progress(links, overwrite) else {
        let err = IpcResponse::Error {
            code: "SERVICE_UNAVAILABLE".to_string(),
            message: "Service not initialized".to_string(),
        };
        return send_response(stream, &err).await;
    };

    while let Some(msg) = rx.recv().await {
        let is_terminal = matches!(
            msg,
            IpcResponse::ImportResult { .. } | IpcResponse::Error { .. }
        );
        send_response(stream, &msg).await?;
        if is_terminal {
            return Ok(());
        }
    }

    // Channel closed without terminal message - protocol error
    let err = IpcResponse::Error {
        code: "PROTOCOL_ERROR".to_string(),
        message: "Import stream closed unexpectedly".to_string(),
    };
    let _ = send_response(stream, &err).await;
    Err(())
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
                        IpcCommand::ImportLinks {
                            links,
                            overwrite,
                            stream_progress: true,
                        } => {
                            // Streaming import: send progress + final result
                            if handle_streaming_import(&mut stream, links, overwrite)
                                .await
                                .is_err()
                            {
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
