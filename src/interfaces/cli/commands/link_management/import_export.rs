//! Import and export link commands

use colored::Colorize;
use std::path::Path;

use crate::client::LinkClient;
use crate::interfaces::cli::CliError;
use crate::services::ImportLinkItemRich;
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

    // Convert ShortLink to ImportLinkItemRich for the client
    let import_items: Vec<ImportLinkItemRich> = imported_links
        .into_iter()
        .map(|link| ImportLinkItemRich {
            code: link.code,
            target: link.target,
            created_at: link.created_at,
            expires_at: link.expires_at,
            password: link.password,
            click_count: link.click,
            row_num: None,
        })
        .collect();

    let result = client.import_links(import_items, force_overwrite).await?;

    // Print errors if any
    for item in &result.failed_items {
        if item.code.is_empty() {
            println!("{} {}", "✗".bold().red(), item.error.message());
        } else {
            println!(
                "{} {}: {}",
                "✗".bold().red(),
                item.code,
                item.error.message()
            );
        }
    }

    println!();
    println!(
        "{} Success: {}, skipped: {}, failed: {}",
        "Import finished:".bold().green(),
        result.success_count.to_string().green(),
        result.skipped_count.to_string().yellow(),
        result.failed_items.len().to_string().red()
    );

    Ok(())
}
