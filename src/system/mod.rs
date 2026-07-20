//! System-level modules
//!
//! This module contains system-level utilities:
//! - Platform abstraction (signals, locks)
//! - Hot reload functionality
//! - IPC (Inter-Process Communication) for CLI-server communication

pub mod ipc;
pub mod platform;
pub mod reload;
