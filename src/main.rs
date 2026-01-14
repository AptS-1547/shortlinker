//! Shortlinker - A high-performance URL shortener service
//!
//! This application supports multiple execution modes:
//! - **Server mode** (default): Runs as an HTTP server
//! - **CLI mode**: Command-line interface for management
//! - **TUI mode**: Terminal user interface
//!
//! Mode selection is based on command-line arguments and compile-time features.

use clap::Parser;
use dotenv::dotenv;

use shortlinker::cli::{Cli, Commands};
use shortlinker::system::panic_handler::RunMode;

/// Application entry point
///
/// # Mode Selection
/// - `./shortlinker tui` -> TUI mode (if compiled with tui feature)
/// - `./shortlinker <command>` -> CLI mode (if compiled with cli feature)
/// - `./shortlinker` -> Server mode (default, if compiled with server feature)
///
/// # Configuration
/// - `-c <path>` or `--config <path>` -> Use custom configuration file
/// - No config flag -> Use default "config.toml" if it exists
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv().ok();

    // Parse command-line arguments using clap
    let cli = Cli::parse();

    // Determine run mode for panic handler
    let panic_mode = match &cli.command {
        #[cfg(feature = "tui")]
        Some(Commands::Tui) => RunMode::Tui,
        Some(_) => RunMode::Cli,
        None => RunMode::Server,
    };
    shortlinker::system::panic_handler::install_panic_hook(panic_mode);

    // Initialize configuration system
    shortlinker::config::init_config(cli.config);
    let config = shortlinker::config::get_config();

    // Run appropriate mode based on command
    match cli.command {
        #[cfg(feature = "tui")]
        Some(Commands::Tui) => {
            if let Err(e) = shortlinker::runtime::modes::run_tui().await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        Some(cmd) => {
            #[cfg(feature = "cli")]
            {
                if let Err(e) = shortlinker::interfaces::cli::run_cli_command(cmd).await {
                    eprintln!("{}", e.format_colored());
                    std::process::exit(1);
                }
            }

            #[cfg(not(feature = "cli"))]
            {
                let _ = cmd;
                eprintln!("Error: CLI feature not enabled");
                eprintln!("Please compile with --features cli");
                std::process::exit(1);
            }
        }

        None => {
            #[cfg(feature = "server")]
            {
                // Initialize logging system based on config
                let _log_guard = shortlinker::system::logging::init_logging(&config);

                if let Err(e) = shortlinker::runtime::modes::run_server(&config).await {
                    eprintln!("Server error: {:#}", e);
                    std::process::exit(1);
                }
            }

            #[cfg(not(feature = "server"))]
            {
                eprintln!("Error: No features enabled");
                eprintln!("Please compile with one of: --features server, cli, tui, or full");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
