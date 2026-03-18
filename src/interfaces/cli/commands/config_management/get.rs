//! Config get command

use crate::client::ConfigClient;
use crate::config::definitions::get_def;
use crate::config::schema::get_schema;
use crate::interfaces::cli::CliError;
use colored::Colorize;
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

/// Get a configuration value via ConfigClient
pub async fn config_get(client: &ConfigClient, key: String, json: bool) -> Result<(), CliError> {
    let item = client.get(key).await?;

    // Enrich with definition and schema metadata
    let def = get_def(&item.key);
    let schema = get_schema(&item.key);

    let category = def.map(|d| d.category.to_string()).unwrap_or_default();
    let description = def.map(|d| d.description.to_string()).unwrap_or_default();
    let editable = def.map(|d| d.editable).unwrap_or(true);
    let default_value = if item.is_sensitive {
        "(masked)".to_string()
    } else {
        def.map(|d| (d.default_fn)()).unwrap_or_default()
    };

    let enum_options = schema.and_then(|s| {
        s.enum_options
            .map(|opts| opts.into_iter().map(|o| o.value).collect())
    });

    let display_value = if item.is_sensitive {
        "[REDACTED]".to_string()
    } else {
        item.value.clone()
    };

    let detail = ConfigDetail {
        key: item.key.clone(),
        value: display_value,
        category,
        value_type: item.value_type.to_string(),
        default_value,
        requires_restart: item.requires_restart,
        editable,
        sensitive: item.is_sensitive,
        description,
        enum_options,
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

        println!("{}: {}", "Default".bold(), detail.default_value);

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
