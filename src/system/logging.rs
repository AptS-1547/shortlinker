//! Logging system initialization
//!
//! This module provides functions to initialize the tracing/logging system
//! based on application configuration.

use crate::config::StaticConfig;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling;
use tracing_subscriber;

/// Result of logging initialization
pub struct LoggingInitResult {
    /// Worker guard that must be kept alive for the duration of the program
    pub guard: WorkerGuard,
    /// Warning message if logging failed to initialize as configured (fell back to stdout)
    pub warning: Option<String>,
}

/// Initialize logging system based on configuration
///
/// This sets up the logging system according to the loaded configuration,
/// including file output, log rotation, and formatting.
///
/// **Note**: This should be called only once during application startup,
/// after the configuration has been loaded.
///
/// If file-based logging fails (e.g., permission denied, disk full),
/// the system will fall back to stdout and return a warning message.
///
/// # Arguments
/// * `config` - Application configuration containing logging settings
///
/// # Returns
/// * `LoggingInitResult` - Contains the worker guard and optional warning
pub fn init_logging(config: &StaticConfig) -> LoggingInitResult {
    // Create writer based on config
    let (writer, warning): (Box<dyn std::io::Write + Send + Sync>, Option<String>) = if let Some(
        ref log_file,
    ) =
        config.logging.file
    {
        if !log_file.is_empty() && config.logging.enable_rotation {
            // Use rolling log files
            let dir = std::path::Path::new(log_file)
                .parent()
                .unwrap_or(std::path::Path::new("."));
            let filename = std::path::Path::new(log_file)
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("shortlinker.log"));
            let filename_str = filename.to_str().unwrap_or("shortlinker.log");
            match rolling::Builder::new()
                .rotation(rolling::Rotation::DAILY)
                .filename_prefix(filename_str.trim_end_matches(".log"))
                .filename_suffix("log")
                .max_log_files(config.logging.max_backups as usize)
                .build(dir)
            {
                Ok(appender) => (Box::new(appender), None),
                Err(e) => (
                    Box::new(std::io::stdout()),
                    Some(format!(
                        "Failed to create rolling log appender for '{}': {}. Falling back to stdout.",
                        log_file, e
                    )),
                ),
            }
        } else if !log_file.is_empty() {
            // Non-rotating, append to file
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_file)
            {
                Ok(file) => (Box::new(file), None),
                Err(e) => (
                    Box::new(std::io::stdout()),
                    Some(format!(
                        "Failed to open log file '{}': {}. Falling back to stdout.",
                        log_file, e
                    )),
                ),
            }
        } else {
            // Empty filename, output to console
            (Box::new(std::io::stdout()), None)
        }
    } else {
        // Output to console
        (Box::new(std::io::stdout()), None)
    };

    let (non_blocking_writer, guard) = tracing_appender::non_blocking(writer);
    let filter = tracing_subscriber::EnvFilter::new(config.logging.level.clone());

    let subscriber_builder = tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .with_env_filter(filter)
        .with_level(true)
        .with_ansi(config.logging.file.as_ref().is_none_or(|f| f.is_empty()));

    if config.logging.format == "json" {
        subscriber_builder.json().init();
    } else {
        subscriber_builder.init();
    }

    LoggingInitResult { guard, warning }
}
