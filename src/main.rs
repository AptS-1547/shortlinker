//! Shortlinker - A high-performance URL shortener service
//!
//! This application supports multiple execution modes:
//! - **Server mode** (default): Runs as an HTTP server
//! - **CLI mode**: Command-line interface for management
//! - **TUI mode**: Terminal user interface
//!
//! Mode selection is based on command-line arguments and compile-time features.

use dotenv::dotenv;

// Load all modules from shortlinker lib
use shortlinker::*;

/// Application entry point
///
/// # Mode Selection
/// - `./shortlinker tui` -> TUI mode (if compiled with tui feature)
/// - `./shortlinker <args>` -> CLI mode (if compiled with cli feature)
/// - `./shortlinker` -> Server mode (default, if compiled with server feature)
///
/// # Configuration
/// - `-c <path>` or `--config <path>` -> Use custom configuration file
/// - No config flag -> Use default "config.toml" if it exists
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenv().ok();

    // 1. Parse command-line arguments
    let args: Vec<String> = std::env::args().collect();

    // 2. Detect mode early for panic handler
    let filtered_args = crate::config::args::filter_config_args(&args);
    let mode = crate::runtime::modes::detect_mode(&filtered_args);

    // 3. Install panic hook based on mode
    let panic_mode = match mode {
        #[cfg(feature = "server")]
        crate::runtime::modes::Mode::Server => crate::system::panic_handler::RunMode::Server,
        #[cfg(feature = "cli")]
        crate::runtime::modes::Mode::Cli => crate::system::panic_handler::RunMode::Cli,
        #[cfg(feature = "tui")]
        crate::runtime::modes::Mode::Tui => crate::system::panic_handler::RunMode::Tui,
        crate::runtime::modes::Mode::Unknown => crate::system::panic_handler::RunMode::Cli,
    };
    crate::system::panic_handler::install_panic_hook(panic_mode);

    // 4. Parse configuration file path
    let config_path = crate::config::args::parse_config_path(&args);

    // 5. Initialize configuration system
    crate::config::init_config(config_path);
    let config = crate::config::get_config();

    // 6. Run appropriate mode
    match mode {
        #[cfg(feature = "tui")]
        crate::runtime::modes::Mode::Tui => {
            if let Err(e) = crate::runtime::modes::run_tui().await {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }

        #[cfg(feature = "cli")]
        crate::runtime::modes::Mode::Cli => {
            if let Err(e) = crate::runtime::modes::run_cli().await {
                #[cfg(feature = "server")]
                eprintln!("{}", e.format_colored());

                #[cfg(not(feature = "server"))]
                eprintln!("Error: {}", e);

                std::process::exit(1);
            }
        }

        #[cfg(feature = "server")]
        crate::runtime::modes::Mode::Server => {
            // Initialize logging system based on config
            let _log_guard = crate::system::logging::init_logging(config);

            if let Err(e) = crate::runtime::modes::run_server(config).await {
                // 尝试转换为 ShortlinkerError 以获得更好的显示
                eprintln!("Server error: {:#}", e);
                std::process::exit(1);
            }
        }

        crate::runtime::modes::Mode::Unknown => {
            eprintln!("Error: No features enabled");
            eprintln!("Please compile with one of: --features server, cli, tui, or full");
            std::process::exit(1);
        }
    }

    Ok(())
}
