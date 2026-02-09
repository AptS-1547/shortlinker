//! Config export/import commands

use super::helpers::notify_config_change;
use crate::config::definitions::{ALL_CONFIGS, get_def};
use crate::config::validators;
use crate::interfaces::cli::CliError;
use crate::storage::ConfigStore;
use crate::system::ipc::{self, ConfigImportItem, ConfigItemData, IpcError, IpcResponse};
use colored::Colorize;
use sea_orm::DatabaseConnection;
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

/// Export configurations to file (IPC-first with direct-DB fallback)
pub async fn config_export(
    db: DatabaseConnection,
    file_path: Option<String>,
) -> Result<(), CliError> {
    // Try IPC first if server is running
    if ipc::is_server_running() {
        match ipc::config_list(None).await {
            Ok(IpcResponse::ConfigListResult { configs }) => {
                return config_export_from_data(&configs, file_path);
            }
            Ok(IpcResponse::Error { code, message }) => {
                return Err(CliError::CommandError(format!("{}: {}", code, message)));
            }
            Err(IpcError::ServerNotRunning) => {
                // Fall through to direct database operation
            }
            Err(e) => {
                return Err(CliError::CommandError(format!("IPC error: {}", e)));
            }
            _ => {
                return Err(CliError::CommandError(
                    "Unexpected response from server".to_string(),
                ));
            }
        }
    }

    // Fallback: Direct database operation
    config_export_direct(db, file_path).await
}

/// Export configs from IPC data to file or stdout
fn config_export_from_data(
    configs: &[ConfigItemData],
    file_path: Option<String>,
) -> Result<(), CliError> {
    let export_configs: Vec<ExportConfig> = configs
        .iter()
        .map(|cfg| ExportConfig {
            key: cfg.key.clone(),
            value: cfg.value.clone(),
            category: Some(cfg.category.clone()),
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

/// Direct database operation for export (fallback when server is not running)
async fn config_export_direct(
    db: DatabaseConnection,
    file_path: Option<String>,
) -> Result<(), CliError> {
    let store = ConfigStore::new(db);
    let all_configs = store
        .get_all()
        .await
        .map_err(|e| CliError::StorageError(e.to_string()))?;

    let mut configs = Vec::new();

    for def in ALL_CONFIGS {
        let value = all_configs
            .get(def.key)
            .map(|item| (*item.value).clone())
            .unwrap_or_else(|| (def.default_fn)());

        configs.push(ExportConfig {
            key: def.key.to_string(),
            value,
            category: Some(def.category.to_string()),
        });
    }

    let export_data = ExportData {
        version: "1.0".to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        configs,
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

/// Import configurations from file (IPC-first with direct-DB fallback)
pub async fn config_import(
    db: DatabaseConnection,
    file_path: String,
    force: bool,
) -> Result<(), CliError> {
    // Read and parse file (shared between IPC and direct paths)
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

    // Validate all configs first (shared validation for both paths)
    let mut valid_configs = Vec::new();
    let mut skipped = Vec::new();
    let mut invalid = Vec::new();

    for cfg in &import_data.configs {
        // Check if key exists
        let def = match get_def(&cfg.key) {
            Some(d) => d,
            None => {
                skipped.push((cfg.key.clone(), "Unknown key".to_string()));
                continue;
            }
        };

        // Check if editable
        if !def.editable {
            skipped.push((cfg.key.clone(), "Read-only".to_string()));
            continue;
        }

        // Validate value
        if let Err(e) = validators::validate_config_value(&cfg.key, &cfg.value) {
            invalid.push((cfg.key.clone(), e));
            continue;
        }

        valid_configs.push((cfg, def.is_sensitive));
    }

    // Show preview
    if !valid_configs.is_empty() {
        println!("\n{}", "Configs to import:".bold());
        for (cfg, is_sensitive) in &valid_configs {
            let display_value = if *is_sensitive {
                "*****".to_string()
            } else {
                cfg.value.clone()
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

    // Try IPC first if server is running
    if ipc::is_server_running() {
        // Build ConfigImportItem list from validated configs
        let import_items: Vec<ConfigImportItem> = valid_configs
            .iter()
            .map(|(cfg, _)| ConfigImportItem {
                key: cfg.key.clone(),
                value: cfg.value.clone(),
            })
            .collect();

        match ipc::config_import(import_items).await {
            Ok(IpcResponse::ConfigImportResult {
                success,
                skipped: ipc_skipped,
                failed,
                errors,
            }) => {
                // Print errors if any
                for error in &errors {
                    println!("{} {}", "✗".bold().red(), error);
                }

                println!(
                    "\n{} Imported {} configurations successfully.",
                    "✓".bold().green(),
                    success
                );

                if ipc_skipped > 0 {
                    println!(
                        "{} {} configurations skipped.",
                        "ℹ".bold().blue(),
                        ipc_skipped
                    );
                }

                if failed > 0 {
                    println!(
                        "{} {} configurations failed to import.",
                        "✗".bold().red(),
                        failed
                    );
                }

                // Server already handled reload, no need to call notify_config_change
                return Ok(());
            }
            Ok(IpcResponse::Error { code, message }) => {
                return Err(CliError::CommandError(format!("{}: {}", code, message)));
            }
            Err(IpcError::ServerNotRunning) => {
                // Fall through to direct database operation
            }
            Err(e) => {
                return Err(CliError::CommandError(format!("IPC error: {}", e)));
            }
            _ => {
                return Err(CliError::CommandError(
                    "Unexpected response from server".to_string(),
                ));
            }
        }
    }

    // Fallback: Direct database operation
    config_import_direct(db, valid_configs).await
}

/// Direct database operation for import (fallback when server is not running)
async fn config_import_direct(
    db: DatabaseConnection,
    valid_configs: Vec<(&ExportConfig, bool)>,
) -> Result<(), CliError> {
    let store = ConfigStore::new(db);
    let mut success = 0;
    let mut failed = 0;

    for (cfg, _) in valid_configs {
        match store.set(&cfg.key, &cfg.value).await {
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
