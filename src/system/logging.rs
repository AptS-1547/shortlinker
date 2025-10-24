//! Logging system initialization
//!
//! This module provides functions to initialize the tracing/logging system
//! based on application configuration.

use super::app_config::AppConfig;
use tracing_appender::rolling;
use tracing_subscriber;

/// Initialize logging system based on configuration
///
/// This sets up the logging system according to the loaded configuration,
/// including file output, log rotation, and formatting.
///
/// **Note**: This should be called only once during application startup,
/// after the configuration has been loaded.
///
/// # Arguments
/// * `config` - Application configuration containing logging settings
///
/// # Returns
/// * `WorkerGuard` - Must be kept alive for the duration of the program
///   to ensure non-blocking log writes are flushed
///
/// # Panics
/// * If creating the log appender fails
/// * If setting the global subscriber fails (e.g., already initialized)
pub fn init_logging(config: &AppConfig) -> tracing_appender::non_blocking::WorkerGuard {
    // Create writer based on config
    let writer: Box<dyn std::io::Write + Send + Sync> =
        if let Some(ref log_file) = config.logging.file {
            if !log_file.is_empty() && config.logging.enable_rotation {
                // Use rolling log files
                let dir = std::path::Path::new(log_file)
                    .parent()
                    .unwrap_or(std::path::Path::new("."));
                let filename = std::path::Path::new(log_file)
                    .file_name()
                    .unwrap_or(std::ffi::OsStr::new("shortlinker.log"));
                let filename_str = filename.to_str().unwrap_or("shortlinker.log");
                let appender = rolling::Builder::new()
                    .rotation(rolling::Rotation::DAILY)
                    .filename_prefix(filename_str.trim_end_matches(".log"))
                    .filename_suffix("log")
                    .max_log_files(config.logging.max_backups as usize)
                    .build(dir)
                    .expect("Failed to create rolling log appender");
                Box::new(appender)
            } else if !log_file.is_empty() {
                // Non-rotating, append to file
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .expect("Failed to open log file");
                Box::new(file)
            } else {
                // Empty filename, output to console
                Box::new(std::io::stdout())
            }
        } else {
            // Output to console
            Box::new(std::io::stdout())
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

    guard
}
