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
        use std::io::{Read, Write};
        use std::time::Duration;

        use crate::system::ipc::protocol::encode;
        use crate::system::ipc::types::IpcCommand;

        let path_str = Self::socket_path();
        let path = Path::new(&path_str);
        if !path.exists() {
            return false;
        }

        let mut stream = match std::os::unix::net::UnixStream::connect(path) {
            Ok(s) => s,
            Err(_) => return false,
        };

        // 设置短超时，避免僵死进程阻塞检测
        let timeout = Duration::from_secs(1);
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));

        // 发送 Ping 命令，验证服务器能实际响应
        let ping_data = match encode(&IpcCommand::Ping) {
            Ok(data) => data,
            Err(_) => return false,
        };

        if stream.write_all(&ping_data).is_err() {
            return false;
        }

        // 读取响应（至少需要 4 字节长度头 + JSON 数据）
        let mut buf = [0u8; 512];
        matches!(stream.read(&mut buf), Ok(n) if n > 4)
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
