//! Shortlinker - A high-performance URL shortener service
//!
//! This application supports multiple execution modes:
//! - **Server mode** (default): Runs as an HTTP server
//! - **CLI mode**: Command-line interface for management
//!
//! Mode selection is based on command-line arguments and compile-time features.

use aster_forge_logging::init_logging;
use aster_forge_panic::PanicHookConfig;
use clap::Parser;

use shortlinker::cli::Cli;

/// Application entry point
///
/// # Mode Selection
/// - `./shortlinker <command>` -> CLI mode (if compiled with cli feature)
/// - `./shortlinker` -> Server mode (default, if compiled with server feature)
///
/// # Configuration
/// Priority: ENV > .env > config.toml > default values
/// - `.env` file in current directory (if exists)
/// - Environment variables with prefix "SL__" override TOML values
/// - Example: SL__SERVER__PORT=9999
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present (before config loading)
    dotenvy::dotenv().ok();

    // Parse command-line arguments using clap
    let cli = Cli::parse();

    aster_forge_panic::install_panic_hook(
        PanicHookConfig::new(
            "shortlinker",
            env!("CARGO_PKG_VERSION"),
            "https://github.com/AptS-1547/shortlinker",
        )
        .with_crash_log_path("crash.log"),
    );

    // Initialize configuration system
    shortlinker::config::init_config();
    let config = shortlinker::config::get_config();

    // Apply CLI socket override if specified
    if let Some(socket_path) = cli.socket {
        shortlinker::config::set_ipc_socket_override(socket_path);
    }

    // Run appropriate mode based on command
    match cli.command {
        Some(cmd) => {
            #[cfg(feature = "cli")]
            {
                if let Err(e) = shortlinker::cli::run_cli_command(cmd).await {
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
                let log_result = init_logging(&config.logging);
                let _log_guard = log_result.guard;
                if let Some(warning) = log_result.warning {
                    eprintln!("Warning: {}", warning);
                }

                if let Err(e) = shortlinker::runtime::run_server().await {
                    eprintln!("Server error: {:#}", e);
                    std::process::exit(1);
                }
            }

            #[cfg(not(feature = "server"))]
            {
                eprintln!("Error: No features enabled");
                eprintln!("Please compile with one of: --features server, cli, or full");
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
