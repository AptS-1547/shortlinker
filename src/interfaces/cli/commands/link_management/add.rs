//! Add link command

use colored::Colorize;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::try_ipc_or_fallback;
use crate::utils::TimeParser;
use crate::utils::generate_random_code;
use crate::utils::password::process_new_password;
use crate::utils::url_validator::validate_url;

pub async fn add_link(
    storage: Arc<SeaOrmStorage>,
    short_code: Option<String>,
    target_url: String,
    force_overwrite: bool,
    expire_time: Option<String>,
    password: Option<String>,
) -> Result<(), CliError> {
    try_ipc_or_fallback!(
        crate::system::ipc::add_link(
            short_code.clone(),
            target_url.clone(),
            force_overwrite,
            expire_time.clone(),
            password.clone(),
        ),
        IpcResponse::LinkCreated { link, generated_code } => {
            // Show generated code info
            if generated_code {
                println!(
                    "{} Generated random code: {}",
                    "ℹ".bold().blue(),
                    link.code.magenta()
                );
            }

            // Show result
            if let Some(expires_at) = &link.expires_at {
                println!(
                    "{} Added short link: {} -> {} (expires: {})",
                    "✓".bold().green(),
                    link.code.cyan(),
                    link.target.blue().underline(),
                    expires_at.yellow()
                );
            } else {
                println!(
                    "{} Added short link: {} -> {}",
                    "✓".bold().green(),
                    link.code.cyan(),
                    link.target.blue().underline()
                );
            }
            return Ok(());
        },
        @fallback add_link_direct(
            storage,
            short_code,
            target_url,
            force_overwrite,
            expire_time,
            password,
        )
        .await
    )
}

/// Direct database operation (fallback when server is not running)
async fn add_link_direct(
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

    let rt = crate::config::get_runtime_config();
    let random_code_length = rt.get_usize_or(crate::config::keys::FEATURES_RANDOM_CODE_LENGTH, 6);

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
        let dt =
            TimeParser::parse_expire_time_with_help(&expire).map_err(CliError::CommandError)?;
        println!(
            "{} Expiration parsed as: {}",
            "ℹ".bold().blue(),
            dt.format("%Y-%m-%d %H:%M:%S UTC").to_string().yellow()
        );
        Some(dt)
    } else {
        None
    };

    // Process password (hash if needed)
    let hashed_password = process_new_password(password.as_deref())
        .map_err(|e| CliError::CommandError(e.to_string()))?;

    let link = ShortLink {
        code: final_short_code.clone(),
        target: target_url.clone(),
        created_at: chrono::Utc::now(),
        expires_at,
        password: hashed_password,
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

    // Note: No reload notification needed when server is not running
    // The server will load data from database when it starts

    Ok(())
}
