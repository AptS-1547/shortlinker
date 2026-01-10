//! Command-line argument parsing
//!
//! This module provides utilities for parsing command-line arguments,
//! particularly for extracting the configuration file path.

/// Parse configuration file path from command-line arguments
///
/// Supports multiple formats:
/// - `-c path` / `--config path`
/// - `-c=path` / `--config=path`
///
/// # Arguments
/// * `args` - The command-line arguments (including program name at index 0)
///
/// # Returns
/// * `Some(String)` - The configuration file path if found
/// * `None` - No configuration file specified
///
/// # Examples
/// ```
/// use shortlinker::config::args::parse_config_path;
/// let args = vec!["program".to_string(), "-c".to_string(), "custom.toml".to_string()];
/// assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
/// ```
pub fn parse_config_path(args: &[String]) -> Option<String> {
    let mut i = 1; // Skip program name
    while i < args.len() {
        let arg = &args[i];

        // Check for -c or --config with value after space
        if (arg == "-c" || arg == "--config") && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }

        // Check for -c=value or --config=value
        if let Some(path) = arg.strip_prefix("-c=") {
            return Some(path.to_string());
        }
        if let Some(path) = arg.strip_prefix("--config=") {
            return Some(path.to_string());
        }

        i += 1;
    }

    None
}

/// Filter out configuration-related arguments from the argument list
///
/// This removes `-c`/`--config` and their values, so mode detection
/// doesn't get confused by them.
///
/// # Arguments
/// * `args` - The original command-line arguments
///
/// # Returns
/// A new vector with configuration arguments removed
///
/// # Examples
/// ```
/// use shortlinker::config::args::filter_config_args;
/// let args = vec!["program".to_string(), "-c".to_string(), "custom.toml".to_string(), "tui".to_string()];
/// let filtered = filter_config_args(&args);
/// assert_eq!(filtered, vec!["program".to_string(), "tui".to_string()]);
/// ```
pub fn filter_config_args(args: &[String]) -> Vec<String> {
    let mut filtered = Vec::new();
    let mut i = 0;

    while i < args.len() {
        let arg = &args[i];

        // Skip -c or --config and the following value
        if (arg == "-c" || arg == "--config") && i + 1 < args.len() {
            i += 2; // Skip both the flag and its value
            continue;
        }

        // Skip -c=value or --config=value
        if arg.starts_with("-c=") || arg.starts_with("--config=") {
            i += 1;
            continue;
        }

        // Keep this argument
        filtered.push(arg.clone());
        i += 1;
    }

    filtered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_path_short_flag() {
        let args = vec![
            "program".to_string(),
            "-c".to_string(),
            "custom.toml".to_string(),
        ];
        assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
    }

    #[test]
    fn test_parse_config_path_long_flag() {
        let args = vec![
            "program".to_string(),
            "--config".to_string(),
            "custom.toml".to_string(),
        ];
        assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
    }

    #[test]
    fn test_parse_config_path_short_equals() {
        let args = vec!["program".to_string(), "-c=custom.toml".to_string()];
        assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
    }

    #[test]
    fn test_parse_config_path_long_equals() {
        let args = vec!["program".to_string(), "--config=custom.toml".to_string()];
        assert_eq!(parse_config_path(&args), Some("custom.toml".to_string()));
    }

    #[test]
    fn test_parse_config_path_none() {
        let args = vec!["program".to_string(), "tui".to_string()];
        assert_eq!(parse_config_path(&args), None);
    }

    #[test]
    fn test_filter_config_args_short_flag() {
        let args = vec![
            "program".to_string(),
            "-c".to_string(),
            "custom.toml".to_string(),
            "tui".to_string(),
        ];
        let filtered = filter_config_args(&args);
        assert_eq!(filtered, vec!["program".to_string(), "tui".to_string()]);
    }

    #[test]
    fn test_filter_config_args_equals() {
        let args = vec![
            "program".to_string(),
            "--config=custom.toml".to_string(),
            "tui".to_string(),
        ];
        let filtered = filter_config_args(&args);
        assert_eq!(filtered, vec!["program".to_string(), "tui".to_string()]);
    }

    #[test]
    fn test_filter_config_args_no_config() {
        let args = vec!["program".to_string(), "tui".to_string()];
        let filtered = filter_config_args(&args);
        assert_eq!(filtered, vec!["program".to_string(), "tui".to_string()]);
    }
}
