//! System-level modules
//!
//! This module contains system-level functionality:
//! - Configuration management
//! - Lifecycle management (startup, shutdown)
//! - Platform abstraction (signals, locks, reload mechanisms)
//! - Execution mode routing (server, cli, tui)
//! - Command-line argument parsing
//! - Logging system initialization

pub mod app_config;
pub mod args;
pub mod lifetime;
pub mod logging;
pub mod modes;
pub mod platform;
pub mod reload; // Still needed for manual reload_all in admin API
