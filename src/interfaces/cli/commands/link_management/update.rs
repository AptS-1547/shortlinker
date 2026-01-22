//! Update link command

use colored::Colorize;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::try_ipc_or_fallback;
use crate::utils::TimeParser;
use crate::utils::url_validator::validate_url;

pub async fn update_link(
    storage: Arc<SeaOrmStorage>,
    short_code: String,
    target_url: String,
    expire_time: Option<String>,
    password: Option<String>,
) -> Result<(), CliError> {
    try_ipc_or_fallback!(
        crate::system::ipc::update_link(
            short_code.clone(),
            target_url.clone(),
            expire_time.clone(),
            password.clone(),
        ),
        IpcResponse::LinkUpdated { link } => {
            println!(
                "{} Short link updated: {} -> {}",
                "✓".bold().green(),
                link.code.cyan(),
                link.target.blue().underline()
            );
            if let Some(expires_at) = &link.expires_at {
                println!("{} Expiration: {}", "ℹ".bold().blue(), expires_at.yellow());
            }
            return Ok(());
        },
        @fallback update_link_direct(storage, short_code, target_url, expire_time, password).await
    )
}

/// Direct database operation (fallback when server is not running)
async fn update_link_direct(
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
        Ok(Some(link)) => link,
        Ok(None) => {
            return Err(CliError::CommandError(format!(
                "Short link does not exist: {}",
                short_code
            )));
        }
        Err(e) => {
            return Err(CliError::CommandError(format!(
                "Failed to check link: {}",
                e
            )));
        }
    };

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

    Ok(())
}
