//! Config set command

use super::helpers::notify_config_change;
use crate::config::definitions::get_def;
use crate::config::schema::get_schema;
use crate::config::validators;
use crate::interfaces::cli::CliError;
use crate::storage::ConfigStore;
use crate::try_ipc_or_fallback;
use colored::Colorize;
use sea_orm::DatabaseConnection;

/// Set a configuration value (IPC-first with direct-DB fallback)
pub async fn config_set(
    db: DatabaseConnection,
    key: String,
    value: String,
) -> Result<(), CliError> {
    try_ipc_or_fallback!(
        crate::system::ipc::config_set(key.clone(), value.clone()),
        IpcResponse::ConfigSetResult {
            key,
            value,
            requires_restart,
            is_sensitive,
            old_value,
            message,
        } => {
            // Print result
            println!(
                "{} Updated configuration: {} = {}",
                "✓".bold().green(),
                key.cyan(),
                if is_sensitive {
                    "*****".to_string()
                } else {
                    value
                }
            );

            if let Some(old) = old_value
                && !is_sensitive
            {
                println!("  {} {}", "Previous value:".dimmed(), old.dimmed());
            }

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
        @fallback config_set_direct(db, key, value).await
    )
}

/// Direct database operation (fallback when server is not running)
async fn config_set_direct(
    db: DatabaseConnection,
    key: String,
    value: String,
) -> Result<(), CliError> {
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
            "Configuration '{}' is read-only and cannot be modified.",
            key
        )));
    }

    // Validate value
    if let Err(e) = validators::validate_config_value(&key, &value) {
        // Provide helpful error message with allowed values for enum types
        if let Some(schema) = get_schema(&key)
            && let Some(opts) = schema.enum_options
        {
            let allowed: Vec<_> = opts.iter().map(|o| o.value.as_str()).collect();
            return Err(CliError::CommandError(format!(
                "Invalid value '{}' for '{}': {}. Allowed values: {}",
                value,
                key,
                e,
                allowed.join(", ")
            )));
        }
        return Err(CliError::CommandError(format!(
            "Invalid value '{}' for '{}': {}",
            value, key, e
        )));
    }

    // Update in database
    let store = ConfigStore::new(db);
    let result = store
        .set(&key, &value)
        .await
        .map_err(|e| CliError::StorageError(e.to_string()))?;

    // Print result
    println!(
        "{} Updated configuration: {} = {}",
        "✓".bold().green(),
        key.cyan(),
        if def.is_sensitive {
            "*****".to_string()
        } else {
            value.clone()
        }
    );

    if let Some(old) = result.old_value
        && !def.is_sensitive
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
