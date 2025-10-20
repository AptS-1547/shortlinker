//! System-level modules
//!
//! This module contains system-level functionality:
//! - Configuration management
//! - Lifecycle management (startup, shutdown)
//! - Platform abstraction (signals, locks, reload mechanisms)
//! - Execution mode routing (server, cli, tui)

pub mod app_config;
pub mod lifetime;
pub mod modes;
pub mod platform;
// Still needed for manual reload_all in admin API
pub mod reload;
