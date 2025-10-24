//! Mode routing
//!
//! This module provides unified entry points for different execution modes:
//! - Server mode (HTTP server)
//! - CLI mode (Command-line interface)
//! - TUI mode (Terminal UI)
//!
//! The mode selection is based on command-line arguments and feature flags.

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "tui")]
pub mod tui;

// Re-export mode functions for convenience
#[cfg(feature = "server")]
pub use server::run_server;

#[cfg(feature = "cli")]
pub use cli::run_cli;

#[cfg(feature = "tui")]
pub use tui::run_tui;

/// Mode detection result
#[derive(Debug, PartialEq)]
pub enum Mode {
    #[cfg(feature = "server")]
    Server,
    #[cfg(feature = "cli")]
    Cli,
    #[cfg(feature = "tui")]
    Tui,
    Unknown,
}

/// Detect which mode to run based on command-line arguments
///
/// # Mode Detection Logic
/// 1. If "tui" is the first argument and TUI feature is enabled -> TUI mode
/// 2. If there are any arguments and CLI feature is enabled -> CLI mode
/// 3. If server feature is enabled -> Server mode (default)
/// 4. Otherwise -> Unknown (no features enabled)
pub fn detect_mode(args: &[String]) -> Mode {
    #[cfg(feature = "tui")]
    if args.len() > 1 && args[1] == "tui" {
        return Mode::Tui;
    }

    #[cfg(feature = "cli")]
    if args.len() > 1 {
        return Mode::Cli;
    }

    #[cfg(feature = "server")]
    return Mode::Server;

    #[cfg(not(feature = "server"))]
    Mode::Unknown
}
