//! Windows Named Pipe IPC implementation
//!
//! Uses Named Pipes for IPC on Windows systems.
//! Named pipes are created with owner-only ACL (equivalent to Unix 0o600).

use std::ffi::c_void;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::windows::named_pipe::{
    ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions,
};

use super::IpcPlatform;
use crate::config::get_config;

/// RAII guard for security descriptor allocated via SDDL.
/// Ensures `LocalFree` is called when the guard is dropped.
struct SecurityDescriptorGuard {
    sd: *mut c_void,
}

impl Drop for SecurityDescriptorGuard {
    fn drop(&mut self) {
        if !self.sd.is_null() {
            unsafe {
                windows_sys::Win32::Foundation::LocalFree(self.sd);
            }
        }
    }
}

/// Create a Named Pipe with owner-only ACL.
///
/// Uses SDDL string `"D:(A;;GA;;;OW)"` to restrict access to the pipe owner,
/// equivalent to Unix file permission `0o600`.
fn create_pipe_with_acl(pipe_name: &str, first_instance: bool) -> io::Result<NamedPipeServer> {
    use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
    use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;

    // SDDL: Only Owner has GENERIC_ALL access
    let sddl: Vec<u16> = "D:(A;;GA;;;OW)\0".encode_utf16().collect();

    let mut sd_ptr: *mut c_void = std::ptr::null_mut();

    let ret = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1, // SDDL_REVISION_1
            &mut sd_ptr,
            std::ptr::null_mut(),
        )
    };

    if ret == 0 {
        return Err(io::Error::last_os_error());
    }

    let _guard = SecurityDescriptorGuard { sd: sd_ptr };

    let mut sa = SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd_ptr,
        bInheritHandle: 0,
    };

    let server = unsafe {
        ServerOptions::new()
            .first_pipe_instance(first_instance)
            .create_with_security_attributes_raw(
                pipe_name,
                &mut sa as *mut SECURITY_ATTRIBUTES as *mut c_void,
            )?
    };

    tracing::debug!("Named pipe created with owner-only ACL (SDDL: D:(A;;GA;;;OW))");

    Ok(server)
}

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

    fn socket_path() -> String {
        get_config().ipc.effective_socket_path()
    }

    fn is_server_running() -> bool {
        // ERROR_PIPE_BUSY (231): All pipe instances are busy
        // ERROR_FILE_NOT_FOUND (2): Pipe does not exist
        const ERROR_PIPE_BUSY: i32 = 231;

        let pipe_name = Self::socket_path();
        match ClientOptions::new().open(&pipe_name) {
            Ok(_) => true,
            Err(e) => {
                // PIPE_BUSY means server is running (just busy)
                e.raw_os_error() == Some(ERROR_PIPE_BUSY)
            }
        }
    }

    async fn bind() -> io::Result<Self::Listener> {
        let pipe_name = Self::socket_path();
        let server = create_pipe_with_acl(&pipe_name, true)?;

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
        let pipe_name = Self::socket_path();
        let next_server = create_pipe_with_acl(&pipe_name, false)?;
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

        let pipe_name = Self::socket_path();

        for attempt in 0..MAX_RETRIES {
            match ClientOptions::new().open(&pipe_name) {
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
