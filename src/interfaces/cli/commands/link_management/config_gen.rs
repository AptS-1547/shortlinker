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

    // 使用 StaticConfig 生成配置，只包含静态配置项
    let config = crate::config::StaticConfig::default();
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
            println!(
                "  {} {}",
                "Note: Runtime settings (API, routes, features, CORS) are managed via Admin Panel"
                    .dimmed(),
                "".blue()
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
