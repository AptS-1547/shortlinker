//! Windows platform implementation
//!
//! This module provides simplified Windows platform support:
//! - Lock file-based instance management (no process checking)
//! - Lockfile operations
//!
//! Note: Windows implementation is simplified compared to Unix due to
//! lack of signal support.

use crate::system::ipc::platform::{IpcPlatform, PlatformIpc};
use std::fs;
use tracing::{error, info, warn};

use super::PlatformOps;

/// Windows platform operations implementation
pub struct WindowsPlatform;

impl PlatformOps for WindowsPlatform {
    fn init_lockfile() -> std::io::Result<()> {
        use std::io::{self, Write};
        use std::path::Path;

        let lock_file = ".shortlinker.lock";

        // First, check if server is running via IPC (more reliable)
        if PlatformIpc::is_server_running() {
            error!("Server already running (IPC pipe active)");
            error!("You can check with: shortlinker.exe status");
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Server is already running (IPC active)",
            ));
        }

        // Check if lock file already exists (fallback check)
        if Path::new(lock_file).exists() {
            // On Windows, we can't reliably check if the process is still running
            // So we just warn and remove the lock file since IPC check passed
            warn!("Lock file exists but IPC not responding, assuming stale");
            let _ = fs::remove_file(lock_file);
        }

        // Create lock file
        match fs::File::create(lock_file) {
            Ok(mut file) => {
                if let Err(e) = writeln!(file, "Server is running") {
                    error!("Failed to write lock file: {}", e);
                    return Err(e);
                }
            }
            Err(e) => {
                error!("Failed to create lock file: {}", e);
                return Err(e);
            }
        }

        Ok(())
    }

    fn cleanup_lockfile() {
        let lock_file = ".shortlinker.lock";
        if let Err(e) = fs::remove_file(lock_file) {
            error!("Failed to delete lock file: {}", e);
        } else {
            info!("Lock file cleaned: {}", lock_file);
        }

        // Also clean up IPC (no-op on Windows for named pipes)
        PlatformIpc::cleanup();
    }
}

// Export convenience functions for backwards compatibility
pub use WindowsPlatform as Platform;

/// Initialize the lock file
pub fn init_lockfile() -> std::io::Result<()> {
    WindowsPlatform::init_lockfile()
}

/// Clean up the lock file
pub fn cleanup_lockfile() {
    WindowsPlatform::cleanup_lockfile()
}
