//! Unix/Linux platform implementation
//!
//! This module provides Unix/Linux platform support:
//! - PID file management with process checking
//! - Lockfile operations

use crate::system::ipc::platform::{IpcPlatform, PlatformIpc};
use std::fs;
use tracing::{debug, error, info};

use super::PlatformOps;

/// Unix platform operations implementation
pub struct UnixPlatform;

impl PlatformOps for UnixPlatform {
    fn init_lockfile() -> std::io::Result<()> {
        use nix::sys::signal;
        use nix::unistd::Pid;
        use std::path::Path;
        use std::process;

        let pid_file = "shortlinker.pid";

        // First, check if server is running via IPC (more reliable)
        if PlatformIpc::is_server_running() {
            error!("Server already running (IPC socket active)");
            error!("You can check with: ./shortlinker status");
            std::process::exit(1);
        }

        // Clean up any stale IPC socket file
        PlatformIpc::cleanup();

        // Check if PID file already exists (fallback check)
        if Path::new(pid_file).exists() {
            match fs::read_to_string(pid_file) {
                Ok(old_pid_str) => {
                    if let Ok(old_pid) = old_pid_str.trim().parse::<u32>() {
                        let current_pid = process::id();

                        // Docker container restart detection:
                        // If both current and old PID are 1, it's a container restart
                        if current_pid == 1 && old_pid == 1 {
                            info!("Container restart detected, removing old PID file");
                            let _ = fs::remove_file(pid_file);
                        } else if signal::kill(Pid::from_raw(old_pid as i32), None).is_ok() {
                            // Process is still running but IPC is not responding.
                            // This is safer than assuming stale and continuing.
                            error!(
                                "Server already running (PID: {}), but IPC is not responding.",
                                old_pid
                            );
                            error!("The server might be hung or in the process of starting up.");
                            error!("Please stop it manually before restarting: kill {}", old_pid);
                            std::process::exit(1);
                        } else {
                            // Process is dead, clean up stale PID file
                            info!("Stale PID file detected, cleaning up...");
                            let _ = fs::remove_file(pid_file);
                        }
                    }
                }
                Err(_) => {
                    // Corrupted PID file, remove it
                    let _ = fs::remove_file(pid_file);
                }
            }
        }

        // Write current process PID
        let pid = process::id();
        if let Err(e) = fs::write(pid_file, pid.to_string()) {
            error!("Failed to write PID file: {}", e);
            return Err(e);
        } else {
            debug!("Server PID: {}", pid);
        }

        Ok(())
    }

    fn cleanup_lockfile() {
        let pid_file = "shortlinker.pid";
        if let Err(e) = fs::remove_file(pid_file) {
            error!("Failed to delete PID file: {}", e);
        } else {
            info!("PID file cleaned: {}", pid_file);
        }

        // Also clean up IPC socket
        PlatformIpc::cleanup();
        info!("IPC socket cleaned");
    }
}

// Export convenience functions for backwards compatibility

/// Initialize the PID file
pub fn init_lockfile() -> std::io::Result<()> {
    UnixPlatform::init_lockfile()
}

/// Clean up the PID file
pub fn cleanup_lockfile() {
    UnixPlatform::cleanup_lockfile()
}
