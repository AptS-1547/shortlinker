//! Unix Domain Socket IPC implementation
//!
//! Uses Unix Domain Sockets for IPC on Unix-like systems (Linux, macOS).

use std::io;
use std::path::Path;
use tokio::net::{UnixListener, UnixStream};

use super::IpcPlatform;
use crate::config::get_config;

/// Unix IPC implementation using Unix Domain Sockets
pub struct UnixIpc;

impl IpcPlatform for UnixIpc {
    type Stream = UnixStream;
    type Listener = UnixListener;

    fn socket_path() -> String {
        get_config().ipc.effective_socket_path()
    }

    fn is_server_running() -> bool {
        let path_str = Self::socket_path();
        let path = Path::new(&path_str);
        if !path.exists() {
            return false;
        }

        // Try to connect synchronously to verify server is actually running
        // If the socket file exists but server is dead, we'll get ECONNREFUSED
        match std::os::unix::net::UnixStream::connect(path) {
            Ok(_) => true,
            Err(_) => {
                // Socket file exists but can't connect - likely stale
                false
            }
        }
    }

    async fn bind() -> io::Result<Self::Listener> {
        // Clean up any existing stale socket file first
        Self::cleanup();

        let path_str = Self::socket_path();
        let listener = UnixListener::bind(&path_str)?;

        // 设置 socket 文件权限为 0o600（仅属主可读写）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let socket_path = Path::new(&path_str);
            if let Ok(metadata) = std::fs::metadata(socket_path) {
                let mut permissions = metadata.permissions();
                permissions.set_mode(0o600);
                std::fs::set_permissions(socket_path, permissions)?;
                tracing::debug!("IPC socket permissions set to 0600 (owner only)");
            }
        }

        Ok(listener)
    }

    async fn accept(listener: &mut Self::Listener) -> io::Result<Self::Stream> {
        let (stream, _addr) = listener.accept().await?;
        Ok(stream)
    }

    async fn connect() -> io::Result<Self::Stream> {
        UnixStream::connect(&Self::socket_path()).await
    }

    fn cleanup() {
        let _ = std::fs::remove_file(Self::socket_path());
    }
}
