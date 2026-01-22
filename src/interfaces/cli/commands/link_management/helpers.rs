//! Helper functions for link management CLI commands

use colored::Colorize;

use crate::system::ipc::{self, IpcError, IpcResponse};
use crate::system::reload::ReloadTarget;

/// Notify the server to reload data after a link change
///
/// Uses IPC to communicate with the running server.
/// Silently succeeds if the server is not running.
pub async fn notify_data_reload() {
    match ipc::reload(ReloadTarget::Data).await {
        Ok(IpcResponse::ReloadResult {
            success,
            duration_ms,
            message,
            ..
        }) => {
            if success {
                println!(
                    "{} Server cache updated ({}ms)",
                    "✓".bold().green(),
                    duration_ms
                );
            } else {
                println!(
                    "{} Server reload failed: {}",
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
            // Server is not running, this is fine for CLI operations
            // The server will load the data when it starts
        }
        Err(IpcError::Timeout) => {
            println!(
                "{} Server reload timed out (changes saved to database)",
                "⚠".bold().yellow()
            );
        }
        Err(e) => {
            println!("{} Could not notify server: {}", "⚠".bold().yellow(), e);
        }
        _ => {
            // Unexpected response type, ignore
        }
    }
}
