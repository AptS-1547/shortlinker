//! Update link command

use colored::Colorize;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::utils::TimeParser;
use crate::utils::url_validator::validate_url;

pub async fn update_link(
    storage: Arc<SeaOrmStorage>,
    short_code: String,
    target_url: String,
    expire_time: Option<String>,
    password: Option<String>,
) -> Result<(), CliError> {
    // Validate URL format
    if let Err(e) = validate_url(&target_url) {
        return Err(CliError::CommandError(e.to_string()));
    }

    // Check if short code exists
    let old_link = match storage.get(&short_code).await {
        Some(link) => link,
        None => {
            return Err(CliError::CommandError(format!(
                "Short link does not exist: {}",
                short_code
            )));
        }
    };

    let expires_at = if let Some(expire) = expire_time {
        match TimeParser::parse_expire_time(&expire) {
            Ok(dt) => {
                println!(
                    "{} Expiration parsed as: {}",
                    "ℹ".bold().blue(),
                    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
                );
                Some(dt)
            }
            Err(e) => {
                return Err(CliError::CommandError(format!(
                    "Invalid expiration time format: {}. Supported formats:\n  - RFC3339: 2023-10-01T12:00:00Z\n  - Relative time: 1d, 2w, 1y, 1d2h30m",
                    e
                )));
            }
        }
    } else {
        old_link.expires_at // Keep original expiration time
    };
    let updated_link = ShortLink {
        code: short_code.clone(),
        target: target_url.clone(),
        created_at: old_link.created_at, // Keep original creation time
        expires_at,
        password: password.or(old_link.password), // Update password if provided, otherwise keep original
        click: old_link.click,                    // Keep original click count
    };

    storage
        .set(updated_link)
        .await
        .map_err(|e| CliError::CommandError(format!("Failed to update: {}", e)))?;

    println!(
        "{} Short link updated from {} to {}",
        "✓".bold().green(),
        old_link.target.dimmed().underline(),
        target_url.blue().underline()
    );

    if let Some(expire) = expires_at {
        println!(
            "{} Expiration: {}",
            "ℹ".bold().blue(),
            expire.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
        );
    }

    // Notify server to reload
    if let Err(e) = crate::system::platform::notify_server() {
        println!("{} Failed to notify server: {}", "⚠".bold().yellow(), e);
    }

    Ok(())
}
