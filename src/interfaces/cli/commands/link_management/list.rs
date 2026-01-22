//! List links command

use colored::Colorize;
use std::sync::Arc;

use crate::interfaces::cli::CliError;
use crate::storage::SeaOrmStorage;
use crate::system::ipc::ShortLinkData;
use crate::try_ipc_or_fallback;

pub async fn list_links(storage: Arc<SeaOrmStorage>) -> Result<(), CliError> {
    try_ipc_or_fallback!(
        crate::system::ipc::list_links(1, 1000, None),
        IpcResponse::LinkList { links, total, .. } => {
            print_links_ipc(&links, total);
            return Ok(());
        },
        @fallback list_links_direct(storage).await
    )
}

/// Print links from IPC response
fn print_links_ipc(links: &[ShortLinkData], total: usize) {
    if links.is_empty() {
        println!("{} No short links found", "ℹ".bold().blue());
    } else {
        println!("{}", "Short link list:".bold().green());
        println!();
        for link in links {
            let mut info_parts = vec![format!(
                "{} -> {}",
                link.code.cyan(),
                link.target.blue().underline()
            )];

            if let Some(expires_at) = &link.expires_at {
                info_parts.push(
                    format!("(expires: {})", expires_at)
                        .dimmed()
                        .yellow()
                        .to_string(),
                );
            }

            if link.password.is_some() {
                info_parts.push("🔒".to_string());
            }

            if link.click > 0 {
                info_parts.push(
                    format!("(clicks: {})", link.click)
                        .dimmed()
                        .cyan()
                        .to_string(),
                );
            }

            println!("  {}", info_parts.join(" "));
        }
        println!();
        println!(
            "{} Total {} short links",
            "ℹ".bold().blue(),
            total.to_string().green()
        );
    }
}

/// Direct database operation (fallback when server is not running)
async fn list_links_direct(storage: Arc<SeaOrmStorage>) -> Result<(), CliError> {
    let links = storage
        .load_all()
        .await
        .map_err(|e| CliError::CommandError(format!("Failed to load links: {}", e)))?;

    if links.is_empty() {
        println!("{} No short links found", "ℹ".bold().blue());
    } else {
        println!("{}", "Short link list:".bold().green());
        println!();
        for (short_code, link) in &links {
            let mut info_parts = vec![format!(
                "{} -> {}",
                short_code.cyan(),
                link.target.blue().underline()
            )];

            if let Some(expires_at) = link.expires_at {
                info_parts.push(
                    format!("(expires: {})", expires_at.format("%Y-%m-%d %H:%M:%S UTC"))
                        .dimmed()
                        .yellow()
                        .to_string(),
                );
            }

            if link.password.is_some() {
                info_parts.push("🔒".to_string());
            }

            if link.click > 0 {
                info_parts.push(
                    format!("(clicks: {})", link.click)
                        .dimmed()
                        .cyan()
                        .to_string(),
                );
            }

            println!("  {}", info_parts.join(" "));
        }
        println!();
        println!(
            "{} Total {} short links",
            "ℹ".bold().blue(),
            links.len().to_string().green()
        );
    }
    Ok(())
}
