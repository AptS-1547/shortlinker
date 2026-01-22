//! IPC server
//!
//! Runs alongside the HTTP server to handle IPC commands from CLI.

use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, error, info, warn};

use super::handler::handle_command;
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
#[cfg(unix)]
pub async fn start_ipc_server() -> Option<tokio::task::JoinHandle<()>> {
    let handle = tokio::spawn(async move {
        let listener = match PlatformIpc::bind().await {
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
            match PlatformIpc::accept(&listener).await {
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

/// Start the IPC server (Windows version - placeholder)
///
/// Windows Named Pipe support is not fully implemented.
/// On Windows, the IPC server will not start.
#[cfg(windows)]
pub async fn start_ipc_server() -> Option<tokio::task::JoinHandle<()>> {
    warn!("IPC server not fully implemented on Windows");
    warn!("Consider using WSL or the HTTP API for reload operations");
    None
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

                    // Handle the command
                    let response = handle_command(cmd).await;

                    // Send the response
                    match encode(&response) {
                        Ok(data) => {
                            if let Err(e) = stream.write_all(&data).await {
                                error!("Failed to send IPC response: {}", e);
                                return;
                            }
                            if let Err(e) = stream.flush().await {
                                error!("Failed to flush IPC response: {}", e);
                                return;
                            }
                        }
                        Err(e) => {
                            error!("Failed to encode IPC response: {}", e);
                            // Send an error response
                            let error_response = IpcResponse::Error {
                                code: "ENCODE_ERROR".to_string(),
                                message: e.to_string(),
                            };
                            if let Ok(data) = encode(&error_response) {
                                let _ = stream.write_all(&data).await;
                                let _ = stream.flush().await;
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
                    if let Ok(data) = encode(&error_response) {
                        let _ = stream.write_all(&data).await;
                        let _ = stream.flush().await;
                    }
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
