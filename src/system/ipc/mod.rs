//! IPC (Inter-Process Communication) module
//!
//! This module provides IPC-based communication between the server and CLI.
//! It replaces the old signal-based (Unix) and file-polling (Windows) mechanisms.
//!
//! # Architecture
//!
//! - **types.rs**: Protocol type definitions (commands, responses, errors)
//! - **protocol.rs**: Message encoding/decoding (length-prefixed JSON)
//! - **platform/**: Platform-specific implementations (Unix Domain Socket, Named Pipe)
//! - **server.rs**: IPC server that runs alongside the HTTP server
//! - **client.rs**: IPC client for CLI commands
//! - **handler.rs**: Command handler that processes IPC commands
//!
//! # Usage
//!
//! ## Server side
//!
//! ```ignore
//! use crate::system::ipc::server::start_ipc_server;
//!
//! // Start IPC server (typically in startup.rs)
//! start_ipc_server().await;
//! ```
//!
//! ## Client side
//!
//! ```ignore
//! use crate::system::ipc::client;
//! use crate::system::reload::ReloadTarget;
//!
//! // Check if server is running
//! if client::is_server_running() {
//!     // Send a ping
//!     let (version, uptime) = client::ping().await?;
//!
//!     // Trigger a reload
//!     client::reload(ReloadTarget::Data).await?;
//! }
//! ```

pub mod client;
pub mod handler;
pub mod platform;
pub mod protocol;
pub mod server;
pub mod types;

pub use client::{is_server_running, ping, reload, send_command};
pub use platform::PlatformIpc;
pub use types::{IpcCommand, IpcError, IpcResponse};
