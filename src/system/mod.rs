//! System-level modules
//!
//! This module contains system-level utilities:
//! - Platform abstraction (signals, locks)
//! - Logging system initialization
//! - Panic handler
//! - Hot reload functionality

pub mod logging;
pub mod panic_handler;
pub mod platform;
pub mod reload;
