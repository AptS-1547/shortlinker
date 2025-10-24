//! TUI mode
//!
//! This module contains the TUI (Terminal User Interface) mode startup logic.
//! It delegates to the actual TUI implementation.

use crate::runtime::lifetime;

/// Run TUI mode
///
/// This function:
/// 1. Performs pre-startup processing for CLI/TUI modes
/// 2. Delegates to the actual TUI implementation
pub async fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    lifetime::startup::cli_tui_pre_startup().await;
    crate::interfaces::tui::run_tui().await
}
