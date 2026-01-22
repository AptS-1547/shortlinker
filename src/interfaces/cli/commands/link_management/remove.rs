//! Remove link command

use colored::Colorize;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::SeaOrmStorage;
use crate::try_ipc_or_fallback;

pub async fn remove_link(storage: Arc<SeaOrmStorage>, short_code: String) -> Result<(), CliError> {
    try_ipc_or_fallback!(
        crate::system::ipc::remove_link(short_code.clone()),
        IpcResponse::LinkDeleted { code } => {
            println!("{} Deleted short link: {}", "✓".bold().green(), code.cyan());
            return Ok(());
        },
        @fallback remove_link_direct(storage, short_code).await
    )
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
