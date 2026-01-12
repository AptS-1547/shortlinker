//! Import and export link commands

use colored::Colorize;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::{SeaOrmStorage, ShortLink};

pub async fn export_links(
    storage: Arc<SeaOrmStorage>,
    file_path: Option<String>,
) -> Result<(), CliError> {
    let links = storage.load_all().await;

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

    let existing_links = if !force_overwrite {
        storage.load_all().await
    } else {
        std::collections::HashMap::new()
    };

    let mut imported_count = 0;
    let mut skipped_count = 0;
    let mut error_count = 0;

    for imported_link in imported_links {
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

        // Validate URL format
        if !imported_link.target.starts_with("http://")
            && !imported_link.target.starts_with("https://")
        {
            println!(
                "{} Skipping code '{}': invalid URL - {}",
                "✗".bold().red(),
                imported_link.code.cyan(),
                imported_link.target
            );
            error_count += 1;
            continue;
        }

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

    // Notify server to reload
    if imported_count > 0
        && let Err(e) = crate::system::platform::notify_server()
    {
        println!("{} Failed to notify server: {}", "⚠".bold().yellow(), e);
    }

    Ok(())
}
