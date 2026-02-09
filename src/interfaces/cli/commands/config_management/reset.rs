//! Config reset command

use super::helpers::notify_config_change;
use crate::config::definitions::get_def;
use crate::interfaces::cli::CliError;
use crate::storage::ConfigStore;
use crate::try_ipc_or_fallback;
use colored::Colorize;
use sea_orm::DatabaseConnection;

/// Reset a configuration to its default value (IPC-first with direct-DB fallback)
pub async fn config_reset(db: DatabaseConnection, key: String) -> Result<(), CliError> {
    try_ipc_or_fallback!(
        crate::system::ipc::config_reset(key.clone()),
        IpcResponse::ConfigResetResult {
            key,
            value,
            requires_restart,
            is_sensitive,
            message,
        } => {
            // Print result
            println!(
                "{} Reset configuration to default: {} = {}",
                "✓".bold().green(),
                key.cyan(),
                if is_sensitive {
                    "*****".to_string()
                } else {
                    value
                }
            );

            if requires_restart {
                println!(
                    "{} This configuration requires a restart to take effect.",
                    "⚠".bold().yellow()
                );
            }

            if let Some(msg) = message {
                println!("{} {}", "ℹ".bold().blue(), msg);
            }

            // Server already handled reload, no need to call notify_config_change
            return Ok(());
        },
        @fallback config_reset_direct(db, key).await
    )
}

/// Direct database operation (fallback when server is not running)
async fn config_reset_direct(db: DatabaseConnection, key: String) -> Result<(), CliError> {
    // Validate key exists
    let def = get_def(&key).ok_or_else(|| {
        CliError::CommandError(format!(
            "Unknown configuration key: '{}'. Use 'config list' to see all available keys.",
            key
        ))
    })?;

    // Check if editable
    if !def.editable {
        return Err(CliError::CommandError(format!(
            "Configuration '{}' is read-only and cannot be reset.",
            key
        )));
    }

    // Get default value
    let default_value = (def.default_fn)();

    // Update in database
    let store = ConfigStore::new(db);
    let result = store
        .set(&key, &default_value)
        .await
        .map_err(|e| CliError::StorageError(e.to_string()))?;

    // Print result
    println!(
        "{} Reset configuration to default: {} = {}",
        "✓".bold().green(),
        key.cyan(),
        if def.is_sensitive {
            "*****".to_string()
        } else {
            default_value.clone()
        }
    );

    if let Some(old) = result.old_value
        && !def.is_sensitive
        && old != default_value
    {
        println!("  {} {}", "Previous value:".dimmed(), old.dimmed());
    }

    if result.requires_restart {
        println!(
            "{} This configuration requires a restart to take effect.",
            "⚠".bold().yellow()
        );
    }

    // Notify about config change (triggers hot-reload if not requires_restart)
    notify_config_change(result.requires_restart).await;

    Ok(())
}
