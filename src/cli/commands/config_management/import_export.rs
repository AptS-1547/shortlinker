//! Config export/import commands

use super::helpers::notify_config_change;
use crate::cli::CliError;
use crate::client::ConfigClient;
use crate::config::definitions::CONFIG_REGISTRY;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};

#[derive(Serialize, Deserialize)]
struct ExportConfig {
    key: String,
    value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ExportData {
    version: String,
    exported_at: String,
    configs: Vec<ExportConfig>,
}

/// Export configurations to file via ConfigClient
pub async fn config_export(
    client: &ConfigClient,
    file_path: Option<String>,
) -> Result<(), CliError> {
    let items = client.get_all(None).await?;

    let export_configs: Vec<ExportConfig> = items
        .iter()
        .map(|item| {
            let category = CONFIG_REGISTRY
                .get(&item.key)
                .map(|definition| definition.category.to_string());
            ExportConfig {
                key: item.key.clone(),
                value: item.value.clone(),
                category,
            }
        })
        .collect();

    let export_data = ExportData {
        version: "1.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        configs: export_configs,
    };

    let json_str = serde_json::to_string_pretty(&export_data)
        .map_err(|e| CliError::CommandError(format!("Failed to serialize to JSON: {}", e)))?;

    match file_path {
        Some(path) => {
            fs::write(&path, &json_str).map_err(|e| {
                CliError::CommandError(format!("Failed to write to '{}': {}", path, e))
            })?;
            println!(
                "{} Exported {} configurations to {}",
                "✓".bold().green(),
                export_data.configs.len(),
                path.cyan()
            );
        }
        None => {
            // Output to stdout
            println!("{}", json_str);
        }
    }

    Ok(())
}

/// Import configurations from file via ConfigClient
pub async fn config_import(
    client: &ConfigClient,
    file_path: String,
    force: bool,
) -> Result<(), CliError> {
    // Read and parse file
    let content = fs::read_to_string(&file_path)
        .map_err(|e| CliError::CommandError(format!("Failed to read '{}': {}", file_path, e)))?;

    let import_data: ExportData = serde_json::from_str(&content)
        .map_err(|e| CliError::CommandError(format!("Failed to parse JSON: {}", e)))?;

    println!(
        "{} Importing {} configurations from {} (exported at {})...",
        "ℹ".bold().blue(),
        import_data.configs.len(),
        file_path.cyan(),
        import_data.exported_at.dimmed()
    );
    // Validate all configs first
    let mut valid_configs = Vec::new();
    let mut skipped = Vec::new();
    let mut invalid = Vec::new();
    let validation_lookup = std::collections::HashMap::new();

    for cfg in &import_data.configs {
        // Check if key exists
        let definition = match CONFIG_REGISTRY.get(&cfg.key) {
            Some(definition) => definition,
            None => {
                skipped.push((cfg.key.clone(), "Unknown key".to_string()));
                continue;
            }
        };

        let normalized =
            match CONFIG_REGISTRY.normalize_value(&validation_lookup, &cfg.key, &cfg.value) {
                Ok(normalized) => normalized,
                Err(error) => {
                    invalid.push((cfg.key.clone(), error.to_string()));
                    continue;
                }
            };

        valid_configs.push((cfg, definition.is_sensitive, normalized));
    }

    // Show preview
    if !valid_configs.is_empty() {
        println!("\n{}", "Configs to import:".bold());
        for (cfg, is_sensitive, normalized) in &valid_configs {
            let display_value = if *is_sensitive {
                "*****".to_string()
            } else {
                normalized.clone()
            };
            println!("  {} = {}", cfg.key.green(), display_value);
        }
    }

    if !skipped.is_empty() {
        println!("\n{}", "Skipped:".bold().yellow());
        for (key, reason) in &skipped {
            println!("  {} ({})", key.dimmed(), reason);
        }
    }

    if !invalid.is_empty() {
        println!("\n{}", "Invalid:".bold().red());
        for (key, reason) in &invalid {
            println!("  {} ({})", key.red(), reason);
        }
    }
    if valid_configs.is_empty() {
        println!("\n{} No valid configurations to import.", "ℹ".bold().blue());
        return Ok(());
    }

    // Confirm if not forced
    if !force {
        print!(
            "\nProceed with importing {} configurations? [y/N] ",
            valid_configs.len()
        );
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| CliError::CommandError(format!("Failed to read input: {}", e)))?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{} Import cancelled.", "✗".bold().red());
            return Ok(());
        }
    }

    // Apply each config via ConfigClient
    let mut success = 0;
    let mut failed = 0;

    for (cfg, _, normalized) in valid_configs {
        match client.set(cfg.key.clone(), normalized).await {
            Ok(_) => {
                success += 1;
            }
            Err(e) => {
                println!("{} Failed to set '{}': {}", "✗".bold().red(), cfg.key, e);
                failed += 1;
            }
        }
    }

    println!(
        "\n{} Imported {} configurations successfully.",
        "✓".bold().green(),
        success
    );

    if failed > 0 {
        println!(
            "{} {} configurations failed to import.",
            "✗".bold().red(),
            failed
        );
    }

    // Notify about config change (triggers hot-reload)
    // Import may include configs that require restart, so we pass false
    // to always attempt hot-reload for the ones that don't
    notify_config_change(false).await;

    Ok(())
}
