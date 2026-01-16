//! Remove link command

use colored::Colorize;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::SeaOrmStorage;

pub async fn remove_link(storage: Arc<SeaOrmStorage>, short_code: String) -> Result<(), CliError> {
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

    // Notify server to reload
    if let Err(e) = crate::system::platform::notify_server() {
        println!("{} Failed to notify server: {}", "⚠".bold().yellow(), e);
    }

    Ok(())
}
