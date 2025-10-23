pub mod commands;
pub mod parser;

use crate::repository::RepositoryFactory;
use parser::CliParser;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    RepositoryError(String),
    ParseError(String),
    CommandError(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::RepositoryError(msg) => write!(f, "Repository error: {}", msg),
            CliError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CliError::CommandError(msg) => write!(f, "Command error: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<crate::errors::ShortlinkerError> for CliError {
    fn from(err: crate::errors::ShortlinkerError) -> Self {
        CliError::RepositoryError(err.to_string())
    }
}

pub async fn run_cli() -> Result<(), CliError> {
    let repository = RepositoryFactory::create()
        .await
        .map_err(|e| CliError::RepositoryError(e.to_string()))?;
    let parser = CliParser::new();
    let command = parser.parse()?;
    command.execute(repository).await
}
