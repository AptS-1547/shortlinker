//! Configuration management CLI commands
//!
//! Provides commands to manage configurations via ConfigClient,
//! which handles IPC-first with service-fallback internally.

mod config_gen;
mod get;
mod helpers;
mod import_export;
mod list;
mod reset;
mod set;

use crate::cli::ConfigCommands;
use crate::client::ConfigClient;
use crate::interfaces::cli::CliError;

pub use config_gen::config_generate;
pub use get::config_get;
pub use import_export::{config_export, config_import};
pub use list::config_list;
pub use reset::config_reset;
pub use set::config_set;

/// Run a config subcommand
pub async fn run_config_command(
    client: &ConfigClient,
    cmd: ConfigCommands,
) -> Result<(), CliError> {
    match cmd {
        ConfigCommands::Generate { .. } => {
            unreachable!("Generate command is handled before ConfigClient in run_cli_command")
        }
        ConfigCommands::List { category, json } => config_list(client, category, json).await,
        ConfigCommands::Get { key, json } => config_get(client, key, json).await,
        ConfigCommands::Set { key, value } => config_set(client, key, value).await,
        ConfigCommands::Reset { key } => config_reset(client, key).await,
        ConfigCommands::Export { file_path } => config_export(client, file_path).await,
        ConfigCommands::Import { file_path, force } => {
            config_import(client, file_path, force).await
        }
    }
}
