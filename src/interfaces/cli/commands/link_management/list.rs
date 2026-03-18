//! List links command

use colored::Colorize;

use crate::client::LinkClient;
use crate::interfaces::cli::CliError;
use crate::storage::ShortLink;

pub async fn list_links(client: &LinkClient) -> Result<(), CliError> {
    let (links, total) = client.list_links(1, 1000, None).await?;

    if links.is_empty() {
        println!("{} No short links found", "ℹ".bold().blue());
    } else {
        println!("{}", "Short link list:".bold().green());
        println!();
        for link in &links {
            print_link(link);
        }
        println!();
        println!(
            "{} Total {} short links",
            "ℹ".bold().blue(),
            total.to_string().green()
        );
    }

    Ok(())
}

/// Print a single link with colored formatting
fn print_link(link: &ShortLink) {
    let mut info_parts = vec![format!(
        "{} -> {}",
        link.code.cyan(),
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
