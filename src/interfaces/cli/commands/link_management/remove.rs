//! Remove link command

use colored::Colorize;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::SeaOrmStorage;
use crate::system::ipc::{self, IpcError, IpcResponse};

pub async fn remove_link(storage: Arc<SeaOrmStorage>, short_code: String) -> Result<(), CliError> {
    // Try IPC first if server is running
    if ipc::is_server_running() {
        match ipc::remove_link(short_code.clone()).await {
            Ok(IpcResponse::LinkDeleted { code }) => {
                println!("{} Deleted short link: {}", "✓".bold().green(), code.cyan());
                return Ok(());
            }
            Ok(IpcResponse::Error { code, message }) => {
                return Err(CliError::CommandError(format!("{}: {}", code, message)));
            }
            Err(IpcError::ServerNotRunning) => {
                // Fall through to direct database operation
            }
            Err(e) => {
                return Err(CliError::CommandError(format!("IPC error: {}", e)));
            }
            _ => {
                return Err(CliError::CommandError(
                    "Unexpected response from server".to_string(),
                ));
            }
        }
    }

    // Fallback: Direct database operation when server is not running
    remove_link_direct(storage, short_code).await
}

/// Direct database operation (fallback when server is not running)
async fn remove_link_direct(
    storage: Arc<SeaOrmStorage>,
    short_code: String,
) -> Result<(), CliError> {
    // Check if the link exists before attempting to remove
    let exists = storage
        .get(&short_code)
        .await
        .map_err(|e| CliError::CommandError(format!("Failed to check link: {}", e)))?;

    if exists.is_none() {
        return Err(CliError::CommandError(format!(
            "Short link does not exist: {}",
            short_code
        )));
    }

    storage
        .remove(&short_code)
        .await
        .map_err(|e| CliError::CommandError(format!("Failed to delete: {}", e)))?;

    println!(
        "{} Deleted short link: {}",
        "✓".bold().green(),
        short_code.cyan()
    );

    Ok(())
}
