//! Config get command

use crate::config::definitions::get_def;
use crate::config::schema::get_schema;
use crate::interfaces::cli::CliError;
use crate::storage::ConfigStore;
use colored::Colorize;
use sea_orm::DatabaseConnection;
use serde::Serialize;

#[derive(Serialize)]
struct ConfigDetail {
    key: String,
    value: String,
    category: String,
    value_type: String,
    default_value: String,
    requires_restart: bool,
    editable: bool,
    sensitive: bool,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    enum_options: Option<Vec<String>>,
}

/// Get a configuration value
pub async fn config_get(db: DatabaseConnection, key: String, json: bool) -> Result<(), CliError> {
    // Validate key exists
    let def = get_def(&key).ok_or_else(|| {
        CliError::CommandError(format!(
            "Unknown configuration key: '{}'. Use 'config list' to see all available keys.",
            key
        ))
    })?;

    let store = ConfigStore::new(db);
    let item = store
        .get_full(&key)
        .await
        .map_err(|e| CliError::StorageError(e.to_string()))?;

    let value = item
        .map(|i| (*i.value).clone())
        .unwrap_or_else(|| (def.default_fn)());

    // Mask sensitive values
    let display_value = if def.is_sensitive {
        "*****".to_string()
    } else {
        value
    };

    // Get enum options if available
    let enum_options = get_schema(&key).and_then(|s| {
        s.enum_options
            .map(|opts| opts.into_iter().map(|o| o.value).collect())
    });

    if json {
        let detail = ConfigDetail {
            key: def.key.to_string(),
            value: display_value,
            category: def.category.to_string(),
            value_type: def.value_type.to_string(),
            default_value: (def.default_fn)(),
            requires_restart: def.requires_restart,
            editable: def.editable,
            sensitive: def.is_sensitive,
            description: def.description.to_string(),
            enum_options,
        };
        let json_str = serde_json::to_string_pretty(&detail)
            .map_err(|e| CliError::CommandError(format!("Failed to serialize to JSON: {}", e)))?;
        println!("{}", json_str);
    } else {
        println!();
        println!("{}: {}", "Key".bold(), def.key.green());
        println!("{}: {}", "Value".bold(), display_value.white());
        println!("{}: {}", "Type".bold(), def.value_type);
        println!("{}: {}", "Category".bold(), def.category);
        println!("{}: {}", "Description".bold(), def.description);

        if def.is_sensitive {
            println!("{}: (masked)", "Default".bold());
        } else {
            println!("{}: {}", "Default".bold(), (def.default_fn)());
        }

        if def.requires_restart {
            println!("{}: {}", "Requires Restart".bold(), "Yes".yellow());
        }

        if !def.editable {
            println!("{}: {}", "Editable".bold(), "No (readonly)".red());
        }

        if let Some(opts) = enum_options {
            println!("{}: {}", "Allowed Values".bold(), opts.join(", ").cyan());
        }

        println!();
    }

    Ok(())
}
