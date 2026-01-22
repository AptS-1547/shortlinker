//! CLI helpers for configuration management
//!
//! Provides helper functions for CLI commands that modify configurations.

use crate::config::try_get_runtime_config;
use crate::system::ipc::{self, IpcError, IpcResponse};
use crate::system::reload::ReloadTarget;
use colored::Colorize;

/// Notify about configuration change
///
/// This function should be called after CLI config commands (set/reset/import)
/// to trigger the appropriate reload:
/// - For configs that don't require restart: hot-reload RuntimeConfig via IPC
/// - For configs that require restart: just print a message
///
/// This replaces the old `notify_server()` call which incorrectly triggered
/// data reload (SIGUSR1) instead of config reload.
pub async fn notify_config_change(requires_restart: bool) {
    if requires_restart {
        // Config already printed "requires restart" message, nothing more to do
        return;
    }

    // Try to notify the server via IPC to reload config
    match ipc::reload(ReloadTarget::Config).await {
        Ok(IpcResponse::ReloadResult {
            success,
            duration_ms,
            message,
            ..
        }) => {
            if success {
                println!(
                    "{} Server config reloaded ({}ms)",
                    "✓".bold().green(),
                    duration_ms
                );
            } else {
                println!(
                    "{} Server config reload failed: {}",
                    "⚠".bold().yellow(),
                    message.unwrap_or_default()
                );
            }
        }
        Ok(IpcResponse::Error { code, message }) => {
            println!(
                "{} Server error: {} - {}",
                "⚠".bold().yellow(),
                code,
                message
            );
        }
        Err(IpcError::ServerNotRunning) => {
            // Server is not running, try to hot-reload in CLI process
            if let Some(rc) = try_get_runtime_config() {
                match rc.reload().await {
                    Ok(_) => {
                        println!(
                            "{} Configuration saved (server not running)",
                            "ℹ".bold().blue()
                        );
                    }
                    Err(e) => {
                        println!(
                            "{} Failed to reload config in CLI process: {}",
                            "⚠".bold().yellow(),
                            e
                        );
                    }
                }
            } else {
                println!(
                    "{} Configuration saved (server not running)",
                    "ℹ".bold().blue()
                );
            }
        }
        Err(IpcError::Timeout) => {
            println!("{} Server config reload timed out", "⚠".bold().yellow());
        }
        Err(e) => {
            println!("{} Could not notify server: {}", "⚠".bold().yellow(), e);
        }
        _ => {
            // Unexpected response type, ignore
        }
    }
}
