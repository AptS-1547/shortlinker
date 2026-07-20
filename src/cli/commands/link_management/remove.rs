//! Remove link command

use colored::Colorize;

use crate::cli::CliError;
use crate::client::LinkClient;

pub async fn remove_link(client: &LinkClient, short_code: String) -> Result<(), CliError> {
    client.delete_link(short_code.clone()).await?;

    println!(
        "{} Deleted short link: {}",
        "✓".bold().green(),
        short_code.cyan()
    );

    Ok(())
}
