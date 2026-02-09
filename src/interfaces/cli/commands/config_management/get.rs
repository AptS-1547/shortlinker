//! Config get command

use crate::config::definitions::get_def;
use crate::config::schema::get_schema;
use crate::interfaces::cli::CliError;
use crate::storage::ConfigStore;
use crate::system::ipc::ConfigItemData;
use crate::try_ipc_or_fallback;
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

/// Get a configuration value (IPC-first with direct-DB fallback)
pub async fn config_get(db: DatabaseConnection, key: String, json: bool) -> Result<(), CliError> {
    try_ipc_or_fallback!(
        crate::system::ipc::config_get(key.clone()),
        IpcResponse::ConfigGetResult { config } => {
            print_config_detail_ipc(&config, json)?;
            return Ok(());
        },
        @fallback config_get_direct(db, key, json).await
    )
}

/// Format and print config detail from IPC response
fn print_config_detail_ipc(config: &ConfigItemData, json: bool) -> Result<(), CliError> {
    let detail = ConfigDetail {
        key: config.key.clone(),
        value: config.value.clone(),
        category: config.category.clone(),
        value_type: config.value_type.clone(),
        default_value: config.default_value.clone(),
        requires_restart: config.requires_restart,
        editable: config.editable,
        sensitive: config.sensitive,
        description: config.description.clone(),
        enum_options: config.enum_options.clone(),
    };

    if json {
        let json_str = serde_json::to_string_pretty(&detail)
            .map_err(|e| CliError::CommandError(format!("Failed to serialize to JSON: {}", e)))?;
        println!("{}", json_str);
    } else {
        println!();
        println!("{}: {}", "Key".bold(), detail.key.green());
        println!("{}: {}", "Value".bold(), detail.value.white());
        println!("{}: {}", "Type".bold(), detail.value_type);
        println!("{}: {}", "Category".bold(), detail.category);
        println!("{}: {}", "Description".bold(), detail.description);

        if detail.sensitive {
            println!("{}: (masked)", "Default".bold());
        } else {
            println!("{}: {}", "Default".bold(), detail.default_value);
        }

        if detail.requires_restart {
            println!("{}: {}", "Requires Restart".bold(), "Yes".yellow());
        }

        if !detail.editable {
            println!("{}: {}", "Editable".bold(), "No (readonly)".red());
        }

        if let Some(opts) = &detail.enum_options {
            println!("{}: {}", "Allowed Values".bold(), opts.join(", ").cyan());
        }

        println!();
    }

    Ok(())
}

/// Direct database operation (fallback when server is not running)
async fn config_get_direct(
    db: DatabaseConnection,
    key: String,
    json: bool,
) -> Result<(), CliError> {
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
