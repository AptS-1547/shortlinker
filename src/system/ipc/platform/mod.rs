//! Platform-specific IPC implementations
//!
//! This module provides a unified interface for platform-specific IPC:
//! - Unix: Unix Domain Socket
//! - Windows: Named Pipe

use std::io;
use std::path::Path;
use tokio::io::{AsyncRead, AsyncWrite};

use super::types::SOCKET_PATH_UNIX;

#[cfg(windows)]
use super::types::PIPE_NAME_WINDOWS;

/// Platform-specific IPC operations trait
///
/// This trait defines the interface for platform-specific IPC implementations.
/// Each platform (Unix/Windows) provides its own implementation.
pub trait IpcPlatform: Send + Sync + 'static {
    /// The stream type for this platform
    type Stream: AsyncRead + AsyncWrite + Send + Unpin + 'static;

    /// The listener type for this platform
    type Listener: Send + 'static;

    /// Get the socket/pipe path for this platform
    fn socket_path() -> &'static str;

    /// Check if the server is running by testing socket connectivity
    ///
    /// This performs a quick synchronous check to determine if a server
    /// is listening on the socket.
    fn is_server_running() -> bool;

    /// Create a listener (server side)
    ///
    /// Binds to the socket path and prepares to accept connections.
    fn bind() -> impl std::future::Future<Output = io::Result<Self::Listener>> + Send;

    /// Accept a connection (server side)
    ///
    /// Waits for and accepts a client connection.
    fn accept(
        listener: &Self::Listener,
    ) -> impl std::future::Future<Output = io::Result<Self::Stream>> + Send;

    /// Connect to the server (client side)
    fn connect() -> impl std::future::Future<Output = io::Result<Self::Stream>> + Send;

    /// Clean up the socket file (if applicable)
    ///
    /// On Unix, this removes the socket file.
    /// On Windows, named pipes are automatically cleaned up.
    fn cleanup();
}

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::UnixIpc;
#[cfg(windows)]
pub use windows::WindowsIpc;

/// Platform-specific IPC type alias
///
/// This provides a convenient way to use the platform-specific implementation
/// without conditional compilation at the call site.
#[cfg(unix)]
pub type PlatformIpc = UnixIpc;
#[cfg(windows)]
pub type PlatformIpc = WindowsIpc;

/// Get the socket path for the current platform
pub fn socket_path() -> &'static str {
    #[cfg(unix)]
    return SOCKET_PATH_UNIX;
    #[cfg(windows)]
    return PIPE_NAME_WINDOWS;
}

/// Check if the socket file exists (Unix only)
#[cfg(unix)]
pub fn socket_exists() -> bool {
    Path::new(SOCKET_PATH_UNIX).exists()
}

#[cfg(windows)]
pub fn socket_exists() -> bool {
    // Windows named pipes don't have a filesystem presence
    // We can only check by attempting to connect
    false
}
