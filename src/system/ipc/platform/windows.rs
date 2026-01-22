//! Windows Named Pipe IPC implementation
//!
//! Uses Named Pipes for IPC on Windows systems.

use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};

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
        // ERROR_PIPE_BUSY (231): All pipe instances are busy
        // ERROR_FILE_NOT_FOUND (2): Pipe does not exist
        const ERROR_PIPE_BUSY: i32 = 231;

        match ClientOptions::new().open(PIPE_NAME_WINDOWS) {
            Ok(_) => true,
            Err(e) => {
                // PIPE_BUSY means server is running (just busy)
                e.raw_os_error() == Some(ERROR_PIPE_BUSY)
            }
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

    async fn accept(listener: &mut Self::Listener) -> io::Result<Self::Stream> {
        // Take the current server instance
        let server = listener.server.take().ok_or_else(|| {
            io::Error::new(io::ErrorKind::Other, "Listener has no server instance")
        })?;

        // Wait for a client to connect
        server.connect().await?;

        // Create a new server instance for the next connection
        let next_server = ServerOptions::new()
            .first_pipe_instance(false)
            .create(PIPE_NAME_WINDOWS)?;
        listener.server = Some(next_server);

        // Return the connected stream
        Ok(PipeStream {
            inner: PipeStreamInner::Server(server),
        })
    }

    async fn connect() -> io::Result<Self::Stream> {
        use std::time::Duration;
        use tokio::time::sleep;

        // ERROR_PIPE_BUSY (231): All pipe instances are busy
        const ERROR_PIPE_BUSY: i32 = 231;
        // Retry configuration
        const MAX_RETRIES: u32 = 100;
        const RETRY_DELAY_MS: u64 = 50;

        for attempt in 0..MAX_RETRIES {
            match ClientOptions::new().open(PIPE_NAME_WINDOWS) {
                Ok(client) => {
                    return Ok(PipeStream {
                        inner: PipeStreamInner::Client(client),
                    });
                }
                Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) => {
                    // Pipe is busy, wait and retry
                    if attempt < MAX_RETRIES - 1 {
                        sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
                        continue;
                    }
                    // Max retries reached
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "All pipe instances are busy after retries",
                    ));
                }
                Err(e) => {
                    // Other errors (e.g., pipe not found)
                    return Err(e);
                }
            }
        }

        unreachable!()
    }

    fn cleanup() {
        // Named pipes are automatically cleaned up when all handles are closed
        // No explicit cleanup needed
    }
}
