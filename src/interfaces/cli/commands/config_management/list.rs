//! Config list command

use crate::client::ConfigClient;
use crate::config::definitions::{categories, get_def};
use crate::interfaces::cli::CliError;
use colored::Colorize;
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

/// List all configurations via ConfigClient
pub async fn config_list(
    client: &ConfigClient,
    category: Option<String>,
    json: bool,
) -> Result<(), CliError> {
    let items = client.get_all(category).await?;

    // Group configs by category, enriching with definition metadata
    let mut grouped: BTreeMap<String, Vec<ConfigOutput>> = BTreeMap::new();

    for item in &items {
        let (cat, editable) = if let Some(def) = get_def(&item.key) {
            (def.category.to_string(), def.editable)
        } else {
            ("unknown".to_string(), true)
        };

        let output = ConfigOutput {
            key: item.key.clone(),
            value: item.value.clone(),
            category: cat.clone(),
            value_type: item.value_type.to_string(),
            requires_restart: item.requires_restart,
            editable,
            sensitive: item.is_sensitive,
        };

        grouped.entry(cat).or_default().push(output);
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
