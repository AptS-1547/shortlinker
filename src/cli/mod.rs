//! Command-line parsing and execution for shortlinker.

#[cfg(feature = "cli")]
pub mod commands;

use std::fmt;
#[cfg(feature = "cli")]
use std::sync::Arc;

#[cfg(feature = "cli")]
use crate::client::{ConfigClient, LinkClient, ServiceContext};
#[cfg(feature = "cli")]
use crate::metrics::NoopMetrics;
#[cfg(feature = "cli")]
use crate::storage::StorageFactory;
use clap::{Parser, Subcommand};
#[cfg(feature = "cli")]
use commands::{
    add_link, config_management, export_links, import_links, list_links, remove_link,
    run_reset_password, server_status, update_link,
};

/// Shortlinker command-line arguments.
#[derive(Parser)]
#[command(name = "shortlinker")]
#[command(version)]
#[command(about = "A high-performance URL shortener service", long_about = None)]
pub struct Cli {
    /// Override IPC socket path (Unix) or named pipe path (Windows).
    #[arg(long, short = 's', global = true)]
    pub socket: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available shortlinker commands.
#[derive(Subcommand)]
pub enum Commands {
    /// Add a short link.
    ///
    /// Usage: add [SHORT_CODE] <TARGET_URL>
    Add {
        /// Positional arguments: `[short_code] <target_url>`.
        #[arg(required = true, num_args = 1..=2)]
        args: Vec<String>,

        /// Force overwrite an existing code.
        #[arg(long)]
        force: bool,

        /// Expiration time (RFC3339 or relative, such as `1d` or `2h`).
        #[arg(long)]
        expire: Option<String>,

        /// Password protection.
        #[arg(long)]
        password: Option<String>,
    },

    /// Remove a short link.
    Remove {
        /// Short code to remove.
        short_code: String,
    },

    /// Update a short link.
    Update {
        /// Short code to update.
        short_code: String,

        /// New target URL.
        target_url: String,

        /// New expiration time.
        #[arg(long)]
        expire: Option<String>,

        /// New password.
        #[arg(long)]
        password: Option<String>,
    },

    /// List all short links.
    List,

    /// Export links to a CSV file.
    Export {
        /// Output path. Defaults to a timestamped filename.
        file_path: Option<String>,
    },

    /// Import links from a CSV file.
    Import {
        /// Input file path.
        file_path: String,

        /// Force overwrite existing links.
        #[arg(long)]
        force: bool,
    },

    /// Show server status through IPC.
    Status,

    /// Reset the admin password.
    ResetPassword {
        /// New password. When omitted, prompt interactively.
        #[arg(long)]
        password: Option<String>,

        /// Read the password from stdin.
        #[arg(long)]
        stdin: bool,
    },

    /// Manage configuration.
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

/// Configuration management commands.
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Generate an example configuration file.
    Generate {
        /// Output path. Defaults to `config.example.toml`.
        output_path: Option<String>,

        /// Force overwrite without confirmation.
        #[arg(long)]
        force: bool,
    },

    /// List configuration values.
    List {
        /// Filter by category.
        #[arg(long)]
        category: Option<String>,

        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Get a configuration value.
    Get {
        /// Configuration key.
        key: String,

        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Set a configuration value.
    Set {
        /// Configuration key.
        key: String,

        /// New value.
        value: String,
    },

    /// Reset a configuration value to its default.
    Reset {
        /// Configuration key.
        key: String,
    },

    /// Export configuration values.
    Export {
        /// Output file path. Defaults to stdout.
        file_path: Option<String>,
    },

    /// Import configuration values.
    Import {
        /// Input file path.
        file_path: String,

        /// Force overwrite without confirmation.
        #[arg(long)]
        force: bool,
    },
}

impl Commands {
    /// Parses add-command positional arguments into a short code and target URL.
    pub fn parse_add_args(args: &[String]) -> (Option<String>, String) {
        match args.len() {
            1 => (None, args[0].clone()),
            2 => (Some(args[0].clone()), args[1].clone()),
            _ => unreachable!("clap ensures 1-2 args"),
        }
    }
}

#[derive(Debug)]
pub enum CliError {
    StorageError(String),
    ParseError(String),
    CommandError(String),
}

impl CliError {
    /// Format as simple output
    pub fn format_simple(&self) -> String {
        match self {
            CliError::StorageError(msg) => format!("Storage error: {}", msg),
            CliError::ParseError(msg) => format!("Parse error: {}", msg),
            CliError::CommandError(msg) => format!("Command error: {}", msg),
        }
    }

    /// Format as colored output
    #[cfg(feature = "cli")]
    pub fn format_colored(&self) -> String {
        #[cfg(feature = "server")]
        {
            use colored::Colorize;
            match self {
                CliError::StorageError(msg) => {
                    format!("{} {}", "Storage error:".red().bold(), msg.white())
                }
                CliError::ParseError(msg) => {
                    format!("{} {}", "Parse error:".yellow().bold(), msg.white())
                }
                CliError::CommandError(msg) => {
                    format!("{} {}", "Command error:".red().bold(), msg.white())
                }
            }
        }
        #[cfg(not(feature = "server"))]
        self.format_simple()
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_simple())
    }
}

impl std::error::Error for CliError {}

impl From<crate::errors::ShortlinkerError> for CliError {
    fn from(err: crate::errors::ShortlinkerError) -> Self {
        CliError::StorageError(err.to_string())
    }
}

/// Run a CLI command from clap-parsed input
#[cfg(feature = "cli")]
pub async fn run_cli_command(cmd: Commands) -> Result<(), CliError> {
    // Handle status command separately (uses IPC, no storage needed)
    if let Commands::Status = cmd {
        return server_status().await;
    }

    // Handle reset-password command separately (needs direct DB access)
    if let Commands::ResetPassword { password, stdin } = cmd {
        let storage = StorageFactory::create(NoopMetrics::arc())
            .await
            .map_err(|e| CliError::StorageError(e.to_string()))?;
        run_reset_password(storage.get_db().clone(), password, stdin).await;
        return Ok(());
    }

    // Create shared context for all other commands
    let ctx = Arc::new(ServiceContext::new());
    let link_client = LinkClient::new(ctx.clone());
    let config_client = ConfigClient::new(ctx);

    // Handle config command
    if let Commands::Config { action } = cmd {
        // Generate doesn't need any service, handle it separately
        if let ConfigCommands::Generate { output_path, force } = action {
            return config_management::config_generate(output_path, force).await;
        }

        return config_management::run_config_command(&config_client, action).await;
    }

    match cmd {
        Commands::Add {
            args,
            force,
            expire,
            password,
        } => {
            let (short_code, target_url) = Commands::parse_add_args(&args);
            add_link(
                &link_client,
                short_code,
                target_url,
                force,
                expire,
                password,
            )
            .await
        }

        Commands::Remove { short_code } => remove_link(&link_client, short_code).await,

        Commands::Update {
            short_code,
            target_url,
            expire,
            password,
        } => update_link(&link_client, short_code, target_url, expire, password).await,

        Commands::List => list_links(&link_client).await,

        Commands::Export { file_path } => export_links(&link_client, file_path).await,

        Commands::Import { file_path, force } => import_links(&link_client, file_path, force).await,

        Commands::Status => unreachable!("handled above"),

        Commands::ResetPassword { .. } => unreachable!("handled above"),

        Commands::Config { .. } => unreachable!("handled above"),
    }
}
