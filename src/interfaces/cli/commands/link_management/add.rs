//! Add link command

use colored::Colorize;
use std::sync::Arc;

use super::helpers::notify_data_reload;
use crate::interfaces::cli::CliError;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::utils::TimeParser;
use crate::utils::generate_random_code;
use crate::utils::url_validator::validate_url;

pub async fn add_link(
    storage: Arc<SeaOrmStorage>,
    short_code: Option<String>,
    target_url: String,
    force_overwrite: bool,
    expire_time: Option<String>,
    password: Option<String>,
) -> Result<(), CliError> {
    // Validate URL format
    if let Err(e) = validate_url(&target_url) {
        return Err(CliError::CommandError(e.to_string()));
    }

    let config = crate::config::get_config();
    let random_code_length = config.features.random_code_length;

    let final_short_code = match short_code {
        Some(code) => code,
        None => {
            let code = generate_random_code(random_code_length);
            println!(
                "{} Generated random code: {}",
                "ℹ".bold().blue(),
                code.magenta()
            );
            code
        }
    };

    // Check if short code already exists
    let existing_link = storage
        .get(&final_short_code)
        .await
        .map_err(|e| CliError::CommandError(format!("Failed to check existing link: {}", e)))?;

    if let Some(existing_link) = existing_link {
        if force_overwrite {
            println!(
                "{} Force overwriting code '{}': {} -> {}",
                "⚠".bold().yellow(),
                final_short_code.cyan(),
                existing_link.target.dimmed().underline(),
                target_url.blue()
            );
        } else {
            return Err(CliError::CommandError(format!(
                "Code '{}' already exists and points to {}. Use --force to overwrite",
                final_short_code, existing_link.target
            )));
        }
    }

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
        None
    };
    let link = ShortLink {
        code: final_short_code.clone(),
        target: target_url.clone(),
        created_at: chrono::Utc::now(),
        expires_at,
        password,
        click: 0,
    };

    storage
        .set(link)
        .await
        .map_err(|e| CliError::CommandError(format!("Failed to save: {}", e)))?;

    if let Some(expire) = expires_at {
        println!(
            "{} Added short link: {} -> {} (expires: {})",
            "✓".bold().green(),
            final_short_code.cyan(),
            target_url.blue().underline(),
            expire.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
        );
    } else {
        println!(
            "{} Added short link: {} -> {}",
            "✓".bold().green(),
            final_short_code.cyan(),
            target_url.blue().underline()
        );
    }

    // Notify server to reload via IPC
    notify_data_reload().await;

    Ok(())
}
