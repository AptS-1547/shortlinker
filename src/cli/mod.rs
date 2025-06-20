pub mod commands;
pub mod parser;

use crate::storages::StorageFactory;
use colored::*;
use parser::CliParser;
use std::fmt;
use std::process;

#[derive(Debug)]
pub enum CliError {
    StorageError(String),
    ParseError(String),
    CommandError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            CliError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CliError::CommandError(msg) => write!(f, "Command error: {}", msg),
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
        println!("{} {}", "Error:".bold().red(), e);
        process::exit(1);
    }
}

async fn run_cli_inner() -> Result<(), CliError> {
    let storage = StorageFactory::create()
        .await
        .map_err(|e| CliError::StorageError(e.to_string()))?;
    let parser = CliParser::new();
    let command = parser.parse()?;
    command.execute(storage).await
}
