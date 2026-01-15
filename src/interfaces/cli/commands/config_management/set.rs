//! Config set command

use crate::config::definitions::get_def;
use crate::config::schema::get_schema;
use crate::config::validators;
use crate::interfaces::cli::CliError;
use crate::storage::ConfigStore;
use colored::Colorize;
use sea_orm::DatabaseConnection;

/// Set a configuration value
pub async fn config_set(
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

    // Notify server to reload
    if let Err(e) = crate::system::platform::notify_server() {
        println!("{} Failed to notify server: {}", "⚠".bold().yellow(), e);
    }

    Ok(())
}
