pub mod commands;
pub mod parser;
pub mod process_manager;

use parser::CliParser;
use crate::storages::StorageFactory;
use crate::utils::colors::{BOLD, RED, RESET};
use std::fmt;
use std::process;

#[derive(Debug)]
pub enum CliError {
    StorageError(String),
    ParseError(String),
    CommandError(String),
    ProcessError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            CliError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CliError::CommandError(msg) => write!(f, "Command error: {}", msg),
            CliError::ProcessError(msg) => write!(f, "Process error: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<crate::errors::ShortlinkerError> for CliError {
    fn from(err: crate::errors::ShortlinkerError) -> Self {
        CliError::StorageError(err.to_string())
    }
}

pub async fn run_cli() {
    if let Err(e) = run_cli_inner().await {
        println!("{}{}错误:{} {}", BOLD, RED, RESET, e);
        process::exit(1);
    }
}

async fn run_cli_inner() -> Result<(), CliError> {
    let storage = StorageFactory::create()
        .map_err(|e| CliError::StorageError(e.to_string()))?;
    let parser = CliParser::new();
    let command = parser.parse()?;
    command.execute(storage).await
}
