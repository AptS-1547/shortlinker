//! Generate config command

use colored::Colorize;

use crate::interfaces::cli::CliError;

/// Generate example configuration file
pub async fn generate_config(output_path: Option<String>) -> Result<(), CliError> {
    let path = output_path.unwrap_or_else(|| "config.toml".to_string());

    println!(
        "{} {}",
        "Generating configuration file...".yellow(),
        path.blue()
    );

    let config = crate::config::AppConfig::default();
    match config.save_to_file(&path) {
        Ok(()) => {
            println!(
                "  {} {}",
                "Configuration file generated successfully".green(),
                path.blue()
            );
            println!(
                "  {} {}",
                "Please edit the configuration file and restart the service".yellow(),
                "🔧".blue()
            );
            Ok(())
        }
        Err(e) => {
            println!(
                "  {} {}",
                "Failed to generate configuration file".red(),
                e.to_string().red()
            );
            Err(CliError::CommandError(format!(
                "Unable to write configuration file: {}",
                e
            )))
        }
    }
}
