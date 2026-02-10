//! Import and export link commands

use colored::Colorize;
use std::path::Path;

use crate::client::LinkClient;
use crate::interfaces::cli::CliError;
use crate::services::ImportLinkItem;
use crate::utils::csv_handler;

pub async fn export_links(client: &LinkClient, file_path: Option<String>) -> Result<(), CliError> {
    let links = client.export_links().await?;

    if links.is_empty() {
        println!("{} No short links to export", "ℹ".bold().blue());
        return Ok(());
    }

    let output_path = file_path.unwrap_or_else(csv_handler::generate_export_filename);

    // Convert Vec<ShortLink> to Vec<&ShortLink> for csv_handler
    let link_refs: Vec<&_> = links.iter().collect();
    csv_handler::export_to_csv(&link_refs, &output_path)
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
    client: &LinkClient,
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
    let imported_links = csv_handler::import_from_csv(&file_path)
        .map_err(|e| CliError::CommandError(format!("Failed to import CSV: {}", e)))?;

    if imported_links.is_empty() {
        println!("{} Import file is empty", "ℹ".bold().blue());
        return Ok(());
    }

    // Convert ShortLink to ImportLinkItem for the client
    let import_items: Vec<ImportLinkItem> = imported_links
        .iter()
        .map(|link| ImportLinkItem {
            code: link.code.clone(),
            target: link.target.clone(),
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            password: link.password.clone(),
        })
        .collect();

    let result = client.import_links(import_items, force_overwrite).await?;

    // Print errors if any
    for error in &result.errors {
        if error.code.is_empty() {
            println!("{} {}", "✗".bold().red(), error.message);
        } else {
            println!("{} {}: {}", "✗".bold().red(), error.code, error.message);
        }
    }

    println!();
    println!(
        "{} Success: {}, skipped: {}, failed: {}",
        "Import finished:".bold().green(),
        result.success.to_string().green(),
        result.skipped.to_string().yellow(),
        result.failed.to_string().red()
    );

    Ok(())
}
