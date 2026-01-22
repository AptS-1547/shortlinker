//! CLI helpers for configuration management
//!
//! Provides helper functions for CLI commands that modify configurations.

use crate::config::try_get_runtime_config;
use colored::Colorize;

/// Notify about configuration change
///
/// This function should be called after CLI config commands (set/reset/import)
/// to trigger the appropriate reload:
/// - For configs that don't require restart: hot-reload RuntimeConfig
/// - For configs that require restart: just print a message
///
/// This replaces the old `notify_server()` call which incorrectly triggered
/// data reload (SIGUSR1) instead of config reload.
pub async fn notify_config_change(requires_restart: bool) {
    if requires_restart {
        // Config already printed "requires restart" message, nothing more to do
        return;
    }

    // Try to hot-reload RuntimeConfig
    if let Some(rc) = try_get_runtime_config() {
        match rc.reload().await {
            Ok(_) => {
                println!(
                    "{} Configuration hot-reloaded in this process",
                    "✓".bold().green()
                );
            }
            Err(e) => {
                println!(
                    "{} Failed to hot-reload config in this process: {}",
                    "⚠".bold().yellow(),
                    e
                );
            }
        }
    }

    // Note: We don't call notify_server() here anymore because:
    // 1. It would trigger data reload (SIGUSR1), not config reload
    // 2. The server will pick up config changes on its next RuntimeConfig access
    //    or via HTTP POST /admin/v1/config/reload
}
