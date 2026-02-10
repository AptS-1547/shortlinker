//! Update link command

use colored::Colorize;

use crate::client::LinkClient;
use crate::interfaces::cli::CliError;

pub async fn update_link(
    client: &LinkClient,
    short_code: String,
    target_url: String,
    expire_time: Option<String>,
    password: Option<String>,
) -> Result<(), CliError> {
    let link = client
        .update_link(short_code, target_url, expire_time, password)
        .await?;

    println!(
        "{} Short link updated: {} -> {}",
        "✓".bold().green(),
        link.code.cyan(),
        link.target.blue().underline()
    );

    if let Some(expires_at) = link.expires_at {
        println!(
            "{} Expiration: {}",
            "ℹ".bold().blue(),
            expires_at
                .format("%Y-%m-%d %H:%M:%S UTC")
                .to_string()
                .yellow()
        );
    }

    Ok(())
}
