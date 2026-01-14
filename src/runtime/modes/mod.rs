//! Mode routing
//!
//! This module provides unified entry points for different execution modes:
//! - Server mode (HTTP server)
//! - TUI mode (Terminal UI)
//!
//! CLI mode is now handled directly by clap in main.rs.

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "tui")]
pub mod tui;

// Re-export mode functions for convenience
#[cfg(feature = "server")]
pub use server::run_server;

#[cfg(feature = "tui")]
pub use tui::run_tui;
