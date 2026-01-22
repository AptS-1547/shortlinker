//! Platform abstraction layer
//!
//! This module provides a unified interface for platform-specific operations,
//! abstracting away differences between Unix/Linux and Windows platforms.
//!
//! # Key Features
//! - Lock file management (Unix: PID files, Windows: lock files)
//!
//! # Architecture
//! The platform abstraction is implemented using Rust's conditional compilation:
//! - `unix.rs`: Unix/Linux implementation
//! - `windows.rs`: Windows implementation
//!
//! Upper layers interact with platform operations through the exported functions,
//! which automatically dispatch to the correct platform implementation.

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
}

/// Get the platform name for logging/debugging
pub fn platform_name() -> &'static str {
    #[cfg(unix)]
    return "Unix/Linux";
    #[cfg(windows)]
    return "Windows";
}
