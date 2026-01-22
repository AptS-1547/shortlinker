//! Import and export link commands

use colored::Colorize;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::system::ipc::{self, ImportLinkData, IpcError, IpcResponse};
use crate::utils::password::process_new_password;
use crate::utils::url_validator::validate_url;

pub async fn export_links(
    storage: Arc<SeaOrmStorage>,
    file_path: Option<String>,
) -> Result<(), CliError> {
    // Try IPC first if server is running
    if ipc::is_server_running() {
        match ipc::export_links().await {
            Ok(IpcResponse::ExportResult { links }) => {
                return export_links_to_file(&links, file_path);
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

    // Fallback: Direct database operation when server is not running
    export_links_direct(storage, file_path).await
}

/// Export links to file (shared logic for IPC and direct)
fn export_links_to_file(
    links: &[crate::system::ipc::ShortLinkData],
    file_path: Option<String>,
) -> Result<(), CliError> {
    if links.is_empty() {
        println!("{} No short links to export", "ℹ".bold().blue());
        return Ok(());
    }

    let output_path = file_path.unwrap_or_else(|| {
        format!(
            "shortlinks_export_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        )
    });

    let file = File::create(&output_path).map_err(|e| {
        CliError::CommandError(format!(
            "Failed to create export file '{}': {}",
            output_path, e
        ))
    })?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &links)
        .map_err(|e| CliError::CommandError(format!("Failed to export JSON data: {}", e)))?;

    println!(
        "{} Exported {} short links to: {}",
        "✓".bold().green(),
        links.len().to_string().green(),
        output_path.cyan()
    );

    Ok(())
}

/// Direct database operation (fallback when server is not running)
async fn export_links_direct(
    storage: Arc<SeaOrmStorage>,
    file_path: Option<String>,
) -> Result<(), CliError> {
    let links = storage
        .load_all()
        .await
        .map_err(|e| CliError::CommandError(format!("Failed to load links: {}", e)))?;

    if links.is_empty() {
        println!("{} No short links to export", "ℹ".bold().blue());
        return Ok(());
    }

    // Collect all links
    let links_vec: Vec<&ShortLink> = links.values().collect();

    let output_path = file_path.unwrap_or_else(|| {
        format!(
            "shortlinks_export_{}.json",
            chrono::Utc::now().format("%Y%m%d_%H%M%S")
        )
    });

    let file = File::create(&output_path).map_err(|e| {
        CliError::CommandError(format!(
            "Failed to create export file '{}': {}",
            output_path, e
        ))
    })?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &links_vec)
        .map_err(|e| CliError::CommandError(format!("Failed to export JSON data: {}", e)))?;

    println!(
        "{} Exported {} short links to: {}",
        "✓".bold().green(),
        links.len().to_string().green(),
        output_path.cyan()
    );

    Ok(())
}

pub async fn import_links(
    storage: Arc<SeaOrmStorage>,
    file_path: String,
    force_overwrite: bool,
) -> Result<(), CliError> {
    // Check if file exists
    if !Path::new(&file_path).exists() {
        return Err(CliError::CommandError(format!(
            "Import file not found: {}",
            file_path
        )));
    }

    // Read and parse the import file
    let file = File::open(&file_path).map_err(|e| {
        CliError::CommandError(format!("Failed to open import file '{}': {}", file_path, e))
    })?;
    let reader = BufReader::new(file);
    let imported_links: Vec<ShortLink> = serde_json::from_reader(reader)
        .map_err(|e| CliError::CommandError(format!("Failed to parse JSON file: {}", e)))?;

    if imported_links.is_empty() {
        println!("{} Import file is empty", "ℹ".bold().blue());
        return Ok(());
    }

    // Try IPC first if server is running
    if ipc::is_server_running() {
        // Convert to ImportLinkData
        let import_data: Vec<ImportLinkData> = imported_links
            .iter()
            .map(|link| ImportLinkData {
                code: link.code.clone(),
                target: link.target.clone(),
                expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
                password: link.password.clone(),
            })
            .collect();

        match ipc::import_links(import_data, force_overwrite).await {
            Ok(IpcResponse::ImportResult {
                success,
                skipped,
                failed,
                errors,
            }) => {
                // Print errors if any
                for error in &errors {
                    println!("{} {}", "✗".bold().red(), error);
                }
                println!();
                println!(
                    "{} Success: {}, skipped: {}, failed: {}",
                    "Import finished:".bold().green(),
                    success.to_string().green(),
                    skipped.to_string().yellow(),
                    failed.to_string().red()
                );
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

    // Fallback: Direct database operation when server is not running
    import_links_direct(storage, imported_links, force_overwrite).await
}

/// Direct database operation (fallback when server is not running)
async fn import_links_direct(
    storage: Arc<SeaOrmStorage>,
    imported_links: Vec<ShortLink>,
    force_overwrite: bool,
) -> Result<(), CliError> {
    let existing_links = if !force_overwrite {
        storage
            .load_all()
            .await
            .map_err(|e| CliError::CommandError(format!("Failed to load existing links: {}", e)))?
    } else {
        std::collections::HashMap::new()
    };

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for mut imported_link in imported_links {
        // Check if short code already exists
        if !force_overwrite && existing_links.contains_key(&imported_link.code) {
            println!(
                "{} Skipping existing code: {} (use --force to overwrite)",
                "⚠".bold().yellow(),
                imported_link.code.cyan()
            );
            skipped_count += 1;
            continue;
        }

        // Validate URL using proper validator
        if let Err(e) = validate_url(&imported_link.target) {
            println!(
                "{} Skipping code '{}': {}",
                "✗".bold().red(),
                imported_link.code.cyan(),
                e
            );
            error_count += 1;
            continue;
        }

        // Process password (hash if plaintext, keep if already hashed)
        let processed_password = match process_new_password(imported_link.password.as_deref()) {
            Ok(pwd) => pwd,
            Err(e) => {
                println!(
                    "{} Skipping code '{}': failed to process password - {}",
                    "✗".bold().red(),
                    imported_link.code.cyan(),
                    e
                );
                error_count += 1;
                continue;
            }
        };

        // Update the link with processed password
        imported_link.password = processed_password;

        // Use the imported link directly since it's already a complete ShortLink structure
        match storage.set(imported_link.clone()).await {
            Ok(_) => {
                imported_count += 1;
                println!(
                    "{} Imported: {} -> {}",
                    "✓".bold().green(),
                    imported_link.code.cyan(),
                    imported_link.target.blue().underline()
                );
            }
            Err(e) => {
                println!(
                    "{} Failed to import '{}': {}",
                    "✗".bold().red(),
                    imported_link.code.cyan(),
                    e
                );
                error_count += 1;
            }
        }
    }

    println!();
    println!(
        "{} Success: {} , skipped {} , failed {}",
        "Import finished:".bold().green(),
        imported_count.to_string().green(),
        skipped_count.to_string().yellow(),
        error_count.to_string().red()
    );

    Ok(())
}
