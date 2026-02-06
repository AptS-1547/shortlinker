//! CLI interface module
//!
//! This module provides command-line interface functionality for shortlinker.

pub mod commands;

use crate::cli::{Commands, ConfigCommands};
use crate::storage::StorageFactory;
use commands::{
    add_link, config_management, export_links, import_links, list_links, remove_link,
    run_reset_password, server_status, update_link,
};
use std::fmt;

/// IPC fallback 宏，用于消除 CLI 命令中的 IPC 错误处理重复代码
///
/// 用法:
/// ```ignore
/// try_ipc_or_fallback!(
///     ipc::remove_link(short_code.clone()),
///     IpcResponse::LinkDeleted { code } => {
///         println!("Deleted: {}", code);
///         return Ok(());
///     },
///     @fallback remove_link_direct(storage, short_code).await
/// )
/// ```
#[macro_export]
macro_rules! try_ipc_or_fallback {
    (
        $ipc_call:expr,
        $($pattern:pat => $success_block:block),+ $(,)?,
        @fallback $fallback:expr
    ) => {{
        use $crate::system::ipc::{self, IpcError, IpcResponse};
        use $crate::interfaces::cli::CliError;

        if ipc::is_server_running() {
            match $ipc_call.await {
                $(
                    Ok($pattern) => $success_block
                )+
                Ok(IpcResponse::Error { code, message }) => {
                    return Err(CliError::CommandError(format!("{}: {}", code, message)));
                }
                Err(IpcError::ServerNotRunning) => {
                    // Fall through to fallback
                }
                Err(e) => {
                    return Err(CliError::CommandError(format!("IPC error: {}", e)));
                }
                _ => {
                    return Err(CliError::CommandError(
                        "Unexpected response from server".to_string(),
                    ));
                }
            }
        }

        $fallback
    }};
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
pub async fn run_cli_command(cmd: Commands) -> Result<(), CliError> {
    // Handle status command separately (uses IPC, no storage needed)
    if let Commands::Status = cmd {
        return server_status().await;
    }

    // Handle reset-password command separately (needs DB connection)
    if let Commands::ResetPassword { password, stdin } = cmd {
        let storage = StorageFactory::create()
            .await
            .map_err(|e| CliError::StorageError(e.to_string()))?;
        run_reset_password(storage.get_db().clone(), password, stdin).await;
        return Ok(());
    }

    // Handle config command
    if let Commands::Config { action } = cmd {
        // Generate doesn't need DB connection, handle it separately
        if let ConfigCommands::Generate { output_path, force } = action {
            return config_management::config_generate(output_path, force).await;
        }

        // Other config subcommands need DB connection
        let storage = StorageFactory::create()
            .await
            .map_err(|e| CliError::StorageError(e.to_string()))?;
        return config_management::run_config_command(storage.get_db().clone(), action).await;
    }

    // Create storage for commands that need it
    let storage = StorageFactory::create()
        .await
        .map_err(|e| CliError::StorageError(e.to_string()))?;

    match cmd {
        Commands::Add {
            args,
            force,
            expire,
            password,
        } => {
            let (short_code, target_url) = Commands::parse_add_args(&args);
            add_link(storage, short_code, target_url, force, expire, password).await
        }

        Commands::Remove { short_code } => remove_link(storage, short_code).await,

        Commands::Update {
            short_code,
            target_url,
            expire,
            password,
        } => update_link(storage, short_code, target_url, expire, password).await,

        Commands::List => list_links(storage).await,

        Commands::Export { file_path } => export_links(storage, file_path).await,

        Commands::Import { file_path, force } => import_links(storage, file_path, force).await,

        Commands::Status => unreachable!("handled above"),

        Commands::ResetPassword { .. } => unreachable!("handled above"),

        Commands::Config { .. } => unreachable!("handled above"),

        #[cfg(feature = "tui")]
        Commands::Tui => unreachable!("TUI handled in main"),
    }
}
