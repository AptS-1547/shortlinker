//! Generate config command

use std::io::{self, BufRead, Write};
use std::path::Path;

use colored::Colorize;

use crate::cli::CliError;

/// Generate example configuration file
pub async fn config_generate(output_path: Option<String>, force: bool) -> Result<(), CliError> {
    let path = output_path.unwrap_or_else(|| "config.example.toml".to_string());

    // 检查文件是否存在，非 --force 模式下交互确认
    if !force && Path::new(&path).exists() {
        print!(
            "{} {} {}",
            "File already exists:".yellow(),
            path.blue(),
            "Overwrite? [y/N] ".yellow()
        );
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().lock().read_line(&mut input).unwrap();
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("{}", "Aborted.".red());
            return Ok(());
        }
    }

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
