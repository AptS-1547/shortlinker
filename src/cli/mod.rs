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

impl CliError {
    /// 格式化为简洁输出（CLI 默认不用彩色，保持简洁）
    pub fn format_simple(&self) -> String {
        match self {
            CliError::RepositoryError(msg) => format!("Repository error: {}", msg),
            CliError::ParseError(msg) => format!("Parse error: {}", msg),
            CliError::CommandError(msg) => format!("Command error: {}", msg),
        }
    }

    /// 格式化为彩色输出（可选）
    #[cfg(feature = "cli")]
    pub fn format_colored(&self) -> String {
        #[cfg(feature = "server")]
        {
            use colored::Colorize;
            match self {
                CliError::RepositoryError(msg) => {
                    format!("{} {}", "Repository error:".red().bold(), msg.white())
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
