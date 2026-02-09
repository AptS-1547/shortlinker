//! Config list command

use crate::config::definitions::{ALL_CONFIGS, categories};
use crate::interfaces::cli::CliError;
use crate::storage::ConfigStore;
use crate::system::ipc::ConfigItemData;
use crate::try_ipc_or_fallback;
use colored::Colorize;
use sea_orm::DatabaseConnection;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Serialize)]
struct ConfigOutput {
    key: String,
    value: String,
    category: String,
    value_type: String,
    requires_restart: bool,
    editable: bool,
    #[serde(skip_serializing_if = "is_false")]
    sensitive: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// List all configurations (IPC-first with direct-DB fallback)
pub async fn config_list(
    db: DatabaseConnection,
    category: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    try_ipc_or_fallback!(
        crate::system::ipc::config_list(category.clone()),
        IpcResponse::ConfigListResult { configs } => {
            print_config_list_ipc(&configs, json)?;
            return Ok(());
        },
        @fallback config_list_direct(db, category, json).await
    )
}

/// Format and print config list from IPC response
fn print_config_list_ipc(configs: &[ConfigItemData], json: bool) -> Result<(), CliError> {
    // Convert to ConfigOutput for consistent formatting
    let mut grouped: BTreeMap<String, Vec<ConfigOutput>> = BTreeMap::new();

    for cfg in configs {
        let output = ConfigOutput {
            key: cfg.key.clone(),
            value: cfg.value.clone(),
            category: cfg.category.clone(),
            value_type: cfg.value_type.clone(),
            requires_restart: cfg.requires_restart,
            editable: cfg.editable,
            sensitive: cfg.sensitive,
        };

        grouped
            .entry(cfg.category.clone())
            .or_default()
            .push(output);
    }

    if json {
        let all: Vec<_> = grouped.into_values().flatten().collect();
        let json_str = serde_json::to_string_pretty(&all)
            .map_err(|e| CliError::CommandError(format!("Failed to serialize to JSON: {}", e)))?;
        println!("{}", json_str);
    } else {
        print_grouped_configs(&grouped);
    }

    Ok(())
}

/// Pretty print configs grouped by category
fn print_grouped_configs(grouped: &BTreeMap<String, Vec<ConfigOutput>>) {
    let category_names = [
        (categories::AUTH, "Authentication"),
        (categories::COOKIE, "Cookie"),
        (categories::FEATURES, "Features"),
        (categories::ROUTES, "Routes"),
        (categories::CORS, "CORS"),
        (categories::TRACKING, "Tracking"),
    ];

    for (cat_key, cat_name) in &category_names {
        if let Some(configs) = grouped.get(*cat_key) {
            if configs.is_empty() {
                continue;
            }

            println!("\n{}", format!("[{}]", cat_name).bold().cyan());

            for cfg in configs {
                let mut tags = Vec::new();
                if cfg.sensitive {
                    tags.push("sensitive".yellow().to_string());
                }
                if cfg.requires_restart {
                    tags.push("restart".red().to_string());
                }
                if !cfg.editable {
                    tags.push("readonly".dimmed().to_string());
                }

                let tag_str = if tags.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", tags.join(", "))
                };

                println!("  {} = {}{}", cfg.key.green(), cfg.value.white(), tag_str);
            }
        }
    }
    println!();
}

/// Direct database operation (fallback when server is not running)
async fn config_list_direct(
    db: DatabaseConnection,
    category: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    let store = ConfigStore::new(db);
    let all_configs = store
        .get_all()
        .await
        .map_err(|e| CliError::StorageError(e.to_string()))?;

    // Group configs by category
    let mut grouped: BTreeMap<String, Vec<ConfigOutput>> = BTreeMap::new();

    for def in ALL_CONFIGS {
        // Filter by category if specified
        if let Some(ref cat) = category
            && def.category != cat.as_str()
        {
            continue;
        }

        let value = all_configs
            .get(def.key)
            .map(|item| (*item.value).clone())
            .unwrap_or_else(|| (def.default_fn)());

        // Mask sensitive values
        let display_value = if def.is_sensitive {
            "*****".to_string()
        } else {
            value
        };

        let output = ConfigOutput {
            key: def.key.to_string(),
            value: display_value,
            category: def.category.to_string(),
            value_type: def.value_type.to_string(),
            requires_restart: def.requires_restart,
            editable: def.editable,
            sensitive: def.is_sensitive,
        };

        grouped
            .entry(def.category.to_string())
            .or_default()
            .push(output);
    }

    if json {
        // Flatten for JSON output
        let all: Vec<_> = grouped.into_values().flatten().collect();
        let json_str = serde_json::to_string_pretty(&all)
            .map_err(|e| CliError::CommandError(format!("Failed to serialize to JSON: {}", e)))?;
        println!("{}", json_str);
    } else {
        print_grouped_configs(&grouped);
    }

    Ok(())
}
