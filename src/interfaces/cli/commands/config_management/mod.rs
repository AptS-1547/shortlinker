//! Configuration management CLI commands
//!
//! Provides commands to manage configurations directly from the database
//! when the web admin panel is unavailable.

mod config_gen;
mod get;
mod helpers;
mod import_export;
mod list;
mod reset;
mod set;

use crate::cli::ConfigCommands;
use crate::interfaces::cli::CliError;
use sea_orm::DatabaseConnection;

pub use config_gen::config_generate;
pub use get::config_get;
pub use import_export::{config_export, config_import};
pub use list::config_list;
pub use reset::config_reset;
pub use set::config_set;

/// Run a config subcommand
pub async fn run_config_command(
    db: DatabaseConnection,
    cmd: ConfigCommands,
) -> Result<(), CliError> {
    match cmd {
        ConfigCommands::Generate { .. } => {
            unreachable!("Generate command is handled before DB connection in run_cli_command")
        }
        ConfigCommands::List { category, json } => config_list(db, category, json).await,
        ConfigCommands::Get { key, json } => config_get(db, key, json).await,
        ConfigCommands::Set { key, value } => config_set(db, key, value).await,
        ConfigCommands::Reset { key } => config_reset(db, key).await,
        ConfigCommands::Export { file_path } => config_export(db, file_path).await,
        ConfigCommands::Import { file_path, force } => config_import(db, file_path, force).await,
    }
}
