//! Shortlinker - A high-performance URL shortener service
//!
//! This application supports multiple execution modes:
//! - **Server mode** (default): Runs as an HTTP server
//! - **CLI mode**: Command-line interface for management
//! - **TUI mode**: Terminal user interface
//!
//! Mode selection is based on command-line arguments and compile-time features.

use color_eyre::Result;
use dotenv::dotenv;

// Core modules
mod cache;
mod errors;
mod event;
mod middleware;
mod services;
mod storages;
mod system;
mod utils;

// Optional feature modules
#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "tui")]
mod tui;

/// Application entry point
///
/// # Mode Selection
/// - `./shortlinker tui` -> TUI mode (if compiled with tui feature)
/// - `./shortlinker <args>` -> CLI mode (if compiled with cli feature)
/// - `./shortlinker` -> Server mode (default, if compiled with server feature)
#[actix_web::main]
async fn main() -> Result<(), color_eyre::Report> {
    // Setup global error handling
    color_eyre::install()?;

    // Load environment variables
    dotenv().ok();

    // Initialize configuration system
    crate::system::app_config::init_config();
    let config = crate::system::app_config::get_config();

    // Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // Detect and run appropriate mode
    match system::modes::detect_mode(&args) {
        #[cfg(feature = "tui")]
        system::modes::Mode::Tui => {
            system::modes::run_tui()
                .await
                .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
        }

        #[cfg(feature = "cli")]
        system::modes::Mode::Cli => {
            system::modes::run_cli()
                .await
                .map_err(|e| color_eyre::eyre::eyre!(e.to_string()))?;
        }

        #[cfg(feature = "server")]
        system::modes::Mode::Server => {
            system::modes::run_server(config).await?;
        }

        system::modes::Mode::Unknown => {
            eprintln!("Error: No features enabled");
            eprintln!("Please compile with one of: --features server, cli, tui, or full");
            std::process::exit(1);
        }
    }

    Ok(())
}
