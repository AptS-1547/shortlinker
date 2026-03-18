//! Config reset command

use super::helpers::notify_config_change;
use crate::client::ConfigClient;
use crate::interfaces::cli::CliError;
use colored::Colorize;

/// Reset a configuration to its default value via ConfigClient
pub async fn config_reset(client: &ConfigClient, key: String) -> Result<(), CliError> {
    let result = client.reset(key).await?;

    // Print result
    println!(
        "{} Reset configuration to default: {} = {}",
        "✓".bold().green(),
        result.key.cyan(),
        if result.is_sensitive {
            "*****".to_string()
        } else {
            result.value.clone()
        }
    );

    if result.requires_restart {
        println!(
            "{} This configuration requires a restart to take effect.",
            "⚠".bold().yellow()
        );
    }

    if let Some(msg) = result.message {
        println!("{} {}", "ℹ".bold().blue(), msg);
    }

    // Notify about config change (triggers hot-reload if not requires_restart)
    notify_config_change(result.requires_restart).await;

    Ok(())
}
