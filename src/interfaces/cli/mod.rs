pub mod commands;
pub mod parser;

use crate::storage::StorageFactory;
use commands::{run_reset_password, show_reset_password_help};
use parser::CliParser;
use std::fmt;

#[derive(Debug)]
pub enum CliError {
    StorageError(String),
    ParseError(String),
    CommandError(String),
}

impl CliError {
    /// 格式化为简洁输出（CLI 默认不用彩色，保持简洁）
    pub fn format_simple(&self) -> String {
        match self {
            CliError::StorageError(msg) => format!("Storage error: {}", msg),
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

/// 检查并处理不需要数据库的命令
/// 返回 true 表示命令已处理，false 表示需要继续正常流程
fn handle_standalone_commands() -> bool {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return false;
    }

    // reset-password 需要显示帮助时不需要数据库
    if args[1] == "reset-password" && args.len() < 3 {
        show_reset_password_help();
        std::process::exit(1);
    }

    false
}

pub async fn run_cli() -> Result<(), CliError> {
    // 先处理不需要数据库的命令
    if handle_standalone_commands() {
        return Ok(());
    }

    let storage = StorageFactory::create()
        .await
        .map_err(|e| CliError::StorageError(e.to_string()))?;

    // 处理需要数据库的特殊命令
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "reset-password" {
        run_reset_password(storage.get_db().clone(), &args[2]).await;
        return Ok(());
    }

    let parser = CliParser::new();
    let command = parser.parse()?;
    command.execute(storage).await
}
