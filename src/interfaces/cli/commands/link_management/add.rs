//! Add link command

use colored::Colorize;

use crate::client::LinkClient;
use crate::interfaces::cli::CliError;

pub async fn add_link(
    client: &LinkClient,
    short_code: Option<String>,
    target_url: String,
    force_overwrite: bool,
    expire_time: Option<String>,
    password: Option<String>,
) -> Result<(), CliError> {
    let result = client
        .create_link(
            short_code,
            target_url,
            force_overwrite,
            expire_time,
            password,
        )
        .await?;

    if result.generated_code {
        println!(
            "{} Generated random code: {}",
            "ℹ".bold().blue(),
            result.link.code.magenta()
        );
    }

    if let Some(expires_at) = result.link.expires_at {
        println!(
            "{} Added short link: {} -> {} (expires: {})",
            "✓".bold().green(),
            result.link.code.cyan(),
            result.link.target.blue().underline(),
            expires_at
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string()
                .yellow()
        );
    } else {
        println!(
            "{} Added short link: {} -> {}",
            "✓".bold().green(),
            result.link.code.cyan(),
            result.link.target.blue().underline()
        );
    }

    Ok(())
}
