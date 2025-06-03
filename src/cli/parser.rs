use super::{commands::Command, CliError};
use std::env;

pub struct CliParser;

impl Default for CliParser {
    fn default() -> Self {
        Self::new()
    }
}

impl CliParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse(&self) -> Result<Command, CliError> {
        let args: Vec<String> = env::args().collect();

        if args.len() < 2 {
            return Err(CliError::ParseError("No command provided".to_string()));
        }

        match args[1].as_str() {
            "help" | "--help" | "-h" => Ok(Command::Help),
            "start" => Ok(Command::Start),
            "stop" => Ok(Command::Stop),
            "restart" => Ok(Command::Restart),
            "list" => Ok(Command::List),
            "add" => self.parse_add_command(&args[2..]),
            "remove" => self.parse_remove_command(&args[2..]),
            "update" => self.parse_update_command(&args[2..]),
            "export" => self.parse_export_command(&args[2..]),
            "import" => self.parse_import_command(&args[2..]),
            _ => Err(CliError::ParseError(format!(
                "Unknown command: {}",
                args[1]
            ))),
        }
    }

    pub fn parse_add_command(&self, args: &[String]) -> Result<Command, CliError> {
        if args.is_empty() {
            return Err(CliError::ParseError(
                "Add command requires arguments".to_string(),
            ));
        }

        let mut force_overwrite = false;
        let mut expire_time: Option<String> = None;
        let mut positional_args = Vec::new();

        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--force" => {
                    force_overwrite = true;
                    i += 1;
                }
                "--expire" => {
                    if i + 1 < args.len() {
                        expire_time = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err(CliError::ParseError(
                            "--expire requires a time argument".to_string(),
                        ));
                    }
                }
                _ => {
                    positional_args.push(args[i].clone());
                    i += 1;
                }
            }
        }

        let (short_code, target_url) = match positional_args.len() {
            1 => (None, positional_args[0].clone()), // Random code
            2 => (Some(positional_args[0].clone()), positional_args[1].clone()),
            _ => {
                return Err(CliError::ParseError(
                    "Invalid number of arguments for add command".to_string(),
                ))
            }
        };

        Ok(Command::Add {
            short_code,
            target_url,
            force_overwrite,
            expire_time,
        })
    }

    pub fn parse_remove_command(&self, args: &[String]) -> Result<Command, CliError> {
        if args.len() != 1 {
            return Err(CliError::ParseError(
                "Remove command requires exactly one argument".to_string(),
            ));
        }

        Ok(Command::Remove {
            short_code: args[0].clone(),
        })
    }

    pub fn parse_update_command(&self, args: &[String]) -> Result<Command, CliError> {
        if args.len() < 2 {
            return Err(CliError::ParseError(
                "Update command requires at least two arguments".to_string(),
            ));
        }

        let short_code = args[0].clone();
        let target_url = args[1].clone();
        let mut expire_time = None;

        // Parse optional expire time argument
        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--expire" => {
                    if i + 1 < args.len() {
                        expire_time = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        return Err(CliError::ParseError(
                            "--expire requires a time argument".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(CliError::ParseError(format!(
                        "Unknown parameter: {}",
                        args[i]
                    )));
                }
            }
        }

        Ok(Command::Update {
            short_code,
            target_url,
            expire_time,
        })
    }

    pub fn parse_export_command(&self, args: &[String]) -> Result<Command, CliError> {
        let file_path = if args.is_empty() {
            None
        } else {
            Some(args[0].clone())
        };

        Ok(Command::Export { file_path })
    }

    pub fn parse_import_command(&self, args: &[String]) -> Result<Command, CliError> {
        if args.is_empty() {
            return Err(CliError::ParseError(
                "Import command requires a file path".to_string(),
            ));
        }

        let mut force_overwrite = false;
        let file_path = args[0].clone();

        // 检查是否有 --force 参数
        for arg in args.iter().skip(1) {
            match arg.as_str() {
                "--force" => force_overwrite = true,
                _ => {
                    return Err(CliError::ParseError(format!("Unknown parameter: {}", arg)));
                }
            }
        }

        Ok(Command::Import {
            file_path,
            force_overwrite,
        })
    }
}
