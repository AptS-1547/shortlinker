//! Command-line interface definitions using clap
//!
//! This module defines the CLI structure for shortlinker using clap's derive macros.

use clap::{Parser, Subcommand};

/// Shortlinker - A high-performance URL shortener service
#[derive(Parser)]
#[command(name = "shortlinker")]
#[command(version)]
#[command(about = "A high-performance URL shortener service", long_about = None)]
pub struct Cli {
    /// Override IPC socket path (Unix) or named pipe path (Windows)
    #[arg(long, short = 's', global = true)]
    pub socket: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available commands
#[derive(Subcommand)]
pub enum Commands {
    /// Start TUI mode
    #[cfg(feature = "tui")]
    Tui,

    /// Add a short link
    ///
    /// Usage: add [SHORT_CODE] <TARGET_URL>
    /// - If only URL provided, generates random short code
    /// - If both provided, uses specified short code
    Add {
        /// Positional args: [short_code] <target_url>
        #[arg(required = true, num_args = 1..=2)]
        args: Vec<String>,

        /// Force overwrite existing code
        #[arg(long)]
        force: bool,

        /// Expiration time (RFC3339 or relative like "1d", "2h")
        #[arg(long)]
        expire: Option<String>,

        /// Password protection
        #[arg(long)]
        password: Option<String>,
    },

    /// Remove a short link
    Remove {
        /// Short code to remove
        short_code: String,
    },

    /// Update a short link
    Update {
        /// Short code to update
        short_code: String,

        /// New target URL
        target_url: String,

        /// New expiration time
        #[arg(long)]
        expire: Option<String>,

        /// New password
        #[arg(long)]
        password: Option<String>,
    },

    /// List all short links
    List,

    /// Export links to JSON file
    Export {
        /// Output file path (default: stdout)
        file_path: Option<String>,
    },

    /// Import links from JSON file
    Import {
        /// Input file path
        file_path: String,

        /// Force overwrite existing links
        #[arg(long)]
        force: bool,
    },

    /// Show server status (via IPC)
    Status,

    /// Reset admin password
    ResetPassword {
        /// New password (if not provided, will prompt interactively)
        #[arg(long)]
        password: Option<String>,

        /// Read password from stdin (for scripting)
        #[arg(long)]
        stdin: bool,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigCommands,
    },
}

/// Configuration management commands
#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Generate example configuration file
    Generate {
        /// Output path (default: config.example.toml)
        output_path: Option<String>,

        /// Force overwrite without confirmation
        #[arg(long)]
        force: bool,
    },

    /// List all configurations
    List {
        /// Filter by category (auth, cookie, features, routes, cors, tracking)
        #[arg(long)]
        category: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Get a configuration value
    Get {
        /// Configuration key (e.g., api.admin_token)
        key: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,

        /// New value
        value: String,
    },

    /// Reset configuration to default value
    Reset {
        /// Configuration key
        key: String,
    },

    /// Export configurations to file
    Export {
        /// Output file path (default: stdout)
        file_path: Option<String>,
    },

    /// Import configurations from file
    Import {
        /// Input file path
        file_path: String,

        /// Force overwrite without confirmation
        #[arg(long)]
        force: bool,
    },
}

impl Commands {
    /// Parse add command args into (short_code, target_url)
    pub fn parse_add_args(args: &[String]) -> (Option<String>, String) {
        match args.len() {
            1 => (None, args[0].clone()),
            2 => (Some(args[0].clone()), args[1].clone()),
            _ => unreachable!("clap ensures 1-2 args"),
        }
    }
}
