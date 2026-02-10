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

pub use client::{
    add_link, batch_delete_links, config_get, config_import, config_list, config_reset, config_set,
    export_links, get_link, get_link_stats, import_links, import_links_streaming,
    is_server_running, list_links, ping, reload, remove_link, send_command, update_link,
};
pub use platform::PlatformIpc;
pub use types::{
    ConfigImportItem, ConfigItemData, ImportErrorData, ImportLinkData, ImportPhase, IpcCommand,
    IpcError, IpcResponse, ShortLinkData,
};
