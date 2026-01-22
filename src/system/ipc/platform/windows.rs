//! Windows Named Pipe IPC implementation
//!
//! Uses Named Pipes for IPC on Windows systems.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions};

use super::IpcPlatform;
use crate::system::ipc::types::PIPE_NAME_WINDOWS;

/// Windows IPC implementation using Named Pipes
pub struct WindowsIpc;

/// Wrapper for NamedPipeServer that implements AsyncRead + AsyncWrite
pub struct PipeStream {
    inner: PipeStreamInner,
}

enum PipeStreamInner {
    Server(NamedPipeServer),
    Client(NamedPipeClient),
}

impl AsyncRead for PipeStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match &mut self.inner {
            PipeStreamInner::Server(s) => Pin::new(s).poll_read(cx, buf),
            PipeStreamInner::Client(c) => Pin::new(c).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for PipeStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut self.inner {
            PipeStreamInner::Server(s) => Pin::new(s).poll_write(cx, buf),
            PipeStreamInner::Client(c) => Pin::new(c).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.inner {
            PipeStreamInner::Server(s) => Pin::new(s).poll_flush(cx),
            PipeStreamInner::Client(c) => Pin::new(c).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.inner {
            PipeStreamInner::Server(s) => Pin::new(s).poll_shutdown(cx),
            PipeStreamInner::Client(c) => Pin::new(c).poll_shutdown(cx),
        }
    }
}

/// Wrapper for NamedPipeServer listener
pub struct PipeListener {
    server: Option<NamedPipeServer>,
}

impl IpcPlatform for WindowsIpc {
    type Stream = PipeStream;
    type Listener = PipeListener;

    fn socket_path() -> &'static str {
        PIPE_NAME_WINDOWS
    }

    fn is_server_running() -> bool {
        // Try to connect synchronously
        // Use raw Windows API to check if pipe exists
        match ClientOptions::new().open(PIPE_NAME_WINDOWS) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn bind() -> io::Result<Self::Listener> {
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(PIPE_NAME_WINDOWS)?;

        Ok(PipeListener {
            server: Some(server),
        })
    }

    async fn accept(listener: &Self::Listener) -> io::Result<Self::Stream> {
        // Windows named pipes work differently - each connection requires
        // creating a new pipe instance after accepting
        //
        // For now, we need to handle this specially in the server loop
        // The listener's server instance is used for the first connection

        // This is a simplified implementation that won't work for multiple
        // connections. A proper implementation would need to manage multiple
        // pipe instances.

        // Note: This implementation is incomplete for production use.
        // Windows Named Pipe handling requires more complex logic for
        // multiple concurrent connections.

        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Windows named pipe accept not fully implemented - use Unix socket on WSL",
        ))
    }

    async fn connect() -> io::Result<Self::Stream> {
        let client = ClientOptions::new().open(PIPE_NAME_WINDOWS)?;
        Ok(PipeStream {
            inner: PipeStreamInner::Client(client),
        })
    }

    fn cleanup() {
        // Named pipes are automatically cleaned up when all handles are closed
        // No explicit cleanup needed
    }
}
