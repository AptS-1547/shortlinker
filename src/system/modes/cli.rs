//! CLI mode
//!
//! This module contains the CLI mode startup logic.
//! It delegates to the actual CLI implementation.

use crate::system::lifetime;
use crate::cli::CliError;

/// Run CLI mode
///
/// This function:
/// 1. Performs pre-startup processing for CLI/TUI modes
/// 2. Delegates to the actual CLI implementation
pub async fn run_cli() -> Result<(), CliError> {
    lifetime::startup::cli_tui_pre_startup().await;
    crate::cli::run_cli().await
}
