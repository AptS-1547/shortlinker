//! CLI mode
//!
//! This module contains the CLI mode startup logic.
//! It delegates to the actual CLI implementation.

use crate::system::lifetime;

/// Run CLI mode
///
/// This function:
/// 1. Performs pre-startup processing for CLI/TUI modes
/// 2. Delegates to the actual CLI implementation
pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    lifetime::startup::cli_tui_pre_startup().await;
    crate::cli::run_cli()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })
}
