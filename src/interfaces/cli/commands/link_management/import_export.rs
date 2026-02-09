//! Import and export link commands

use colored::Colorize;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::system::ipc::{self, ImportLinkData, IpcError, IpcResponse, ShortLinkData};
use crate::utils::csv_handler::{self, CsvLinkRow};
use crate::utils::password::process_imported_password;
use crate::utils::url_validator::validate_url;

pub async fn export_links(
    storage: Arc<SeaOrmStorage>,
    file_path: Option<String>,
) -> Result<(), CliError> {
    // Try IPC first if server is running
    if ipc::is_server_running() {
        match export_links_via_ipc().await {
            Ok(links) => {
                return export_links_to_file(&links, file_path);
            }
            Err(ExportIpcError::ServerNotRunning) => {
                // Fall through to direct database operation
            }
            Err(ExportIpcError::Other(msg)) => {
                return Err(CliError::CommandError(msg));
            }
        }
    }

    // Fallback: Direct database operation when server is not running
    export_links_direct(storage, file_path).await
}

/// Error type for paginated IPC export
enum ExportIpcError {
    ServerNotRunning,
    Other(String),
}

/// Export links via IPC using paginated ListLinks to avoid message size limits
async fn export_links_via_ipc() -> Result<Vec<ShortLinkData>, ExportIpcError> {
    let page_size = 100;
    let mut all_links = Vec::new();
    let mut page = 1u64;

    loop {
        match ipc::list_links(page, page_size, None).await {
            Ok(IpcResponse::LinkList { links, total, .. }) => {
                all_links.extend(links);
                if all_links.len() >= total {
                    break;
                }
                page += 1;
            }
            Ok(IpcResponse::Error { code, message }) => {
                return Err(ExportIpcError::Other(format!("{}: {}", code, message)));
            }
            Err(IpcError::ServerNotRunning) => {
                return Err(ExportIpcError::ServerNotRunning);
            }
            Err(e) => {
                return Err(ExportIpcError::Other(format!("IPC error: {}", e)));
            }
            _ => {
                return Err(ExportIpcError::Other(
                    "Unexpected response from server".to_string(),
                ));
            }
        }
    }

    Ok(all_links)
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

    let output_path = file_path.unwrap_or_else(csv_handler::generate_export_filename);

    // Convert ShortLinkData to CsvLinkRow and write CSV
    let file = File::create(&output_path).map_err(|e| {
        CliError::CommandError(format!(
            "Failed to create export file '{}': {}",
            output_path, e
        ))
    })?;
    let writer = BufWriter::new(file);
    let mut csv_writer = csv::WriterBuilder::new().from_writer(writer);

    for link in links {
        let row = CsvLinkRow {
            code: link.code.clone(),
            target: link.target.clone(),
            created_at: link.created_at.clone(),
            expires_at: link.expires_at.clone(),
            password: link.password.clone(),
            click_count: link.click as usize,
        };
        csv_writer
            .serialize(&row)
            .map_err(|e| CliError::CommandError(format!("Failed to write CSV row: {}", e)))?;
    }

    csv_writer
        .flush()
        .map_err(|e| CliError::CommandError(format!("Failed to flush CSV: {}", e)))?;

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

    let output_path = file_path.unwrap_or_else(csv_handler::generate_export_filename);

    // Use csv_handler to export
    csv_handler::export_to_csv(&links_vec, &output_path)
        .map_err(|e| CliError::CommandError(format!("Failed to export CSV: {}", e)))?;

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
    let imported_links: Vec<ShortLink> = csv_handler::import_from_csv(&file_path)
        .map_err(|e| CliError::CommandError(format!("Failed to import CSV: {}", e)))?;

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
        let processed_password = match process_imported_password(imported_link.password.as_deref())
        {
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
