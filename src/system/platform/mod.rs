//! Platform abstraction layer
//!
//! This module provides a unified interface for platform-specific operations,
//! abstracting away differences between Unix/Linux and Windows platforms.
//!
//! # Key Features
//! - Signal/notification mechanisms (Unix: SIGUSR1, Windows: file-based)
//! - Lock file management (Unix: PID files, Windows: lock files)
//! - Reload mechanisms (Unix: signal-based, Windows: polling-based)
//!
//! # Architecture
//! The platform abstraction is implemented using Rust's conditional compilation:
//! - `unix.rs`: Full-featured Unix/Linux implementation
//! - `windows.rs`: Simplified Windows implementation
//!
//! Upper layers interact with platform operations through the exported functions,
//! which automatically dispatch to the correct platform implementation.

use crate::errors::Result;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

// Re-export platform-specific implementations
#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;

/// Platform operations trait
///
/// Defines the interface for platform-specific operations.
/// Each platform provides its own implementation of these methods.
pub trait PlatformOps {
    /// Initialize the lock/PID file
    ///
    /// On Unix: Creates a PID file and checks for existing processes
    /// On Windows: Creates a lock file to prevent multiple instances
    fn init_lockfile() -> std::io::Result<()>;

    /// Clean up the lock/PID file
    ///
    /// Called during shutdown to remove the lock/PID file
    fn cleanup_lockfile();

    /// Notify the server to reload
    ///
    /// On Unix: Sends SIGUSR1 signal to the server process
    /// On Windows: Creates a trigger file that the server polls
    fn notify_server() -> Result<()>;

    /// Setup the reload mechanism
    ///
    /// On Unix: Sets up a signal handler for SIGUSR1
    /// On Windows: Sets up a file polling mechanism
    ///
    /// When triggered, uses the global ReloadCoordinator to reload data.
    fn setup_reload_mechanism() -> impl std::future::Future<Output = ()> + Send;
}

/// Get the platform name for logging/debugging
pub fn platform_name() -> &'static str {
    #[cfg(unix)]
    return "Unix/Linux";
    #[cfg(windows)]
    return "Windows";
}

/// Check if the current platform supports signal-based reload
pub fn supports_signals() -> bool {
    #[cfg(unix)]
    return true;
    #[cfg(windows)]
    return false;
}
