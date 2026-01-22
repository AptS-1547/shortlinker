//! Unix/Linux platform implementation
//!
//! This module provides full-featured Unix/Linux platform support:
//! - PID file management with process checking
//! - SIGUSR1 signal-based reload mechanism
//! - Efficient signal-driven notification

use crate::errors::{Result, ShortlinkerError};
use crate::system::ipc::platform::{IpcPlatform, PlatformIpc};
use crate::system::reload::{ReloadTarget, get_reload_coordinator};
use std::fs;
use tracing::{debug, error, info, warn};

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
                            // Process is still running but IPC check failed
                            // This could mean the process is starting up or shutting down
                            warn!("PID {} exists but IPC not responding, assuming stale", old_pid);
                            let _ = fs::remove_file(pid_file);
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

    fn notify_server() -> Result<()> {
        use nix::sys::signal::{self, Signal};
        use nix::unistd::Pid;

        // Read the PID from file and send SIGUSR1 to the server process
        match fs::read_to_string("shortlinker.pid") {
            Ok(pid_str) => {
                let pid: i32 = pid_str.trim().parse().map_err(|e| {
                    ShortlinkerError::validation(format!("Invalid PID format: {}", e))
                })?;
                signal::kill(Pid::from_raw(pid), Signal::SIGUSR1).map_err(|e| {
                    ShortlinkerError::signal_operation(format!("Failed to send signal: {}", e))
                })?;
                Ok(())
            }
            Err(e) => Err(ShortlinkerError::notify_server(format!(
                "Failed to notify server: {}",
                e
            ))),
        }
    }

    async fn setup_reload_mechanism() {
        use tokio::signal::unix::{SignalKind, signal};

        tokio::spawn(async move {
            // Use match to handle signal creation failure, degrade to disabled signal reload instead of panic
            let mut stream = match signal(SignalKind::user_defined1()) {
                Ok(s) => s,
                Err(e) => {
                    warn!(
                        "Failed to create SIGUSR1 handler: {}. Config reload via signal disabled.",
                        e
                    );
                    return;
                }
            };

            while (stream.recv().await).is_some() {
                info!("Received SIGUSR1, triggering data reload...");

                if let Some(coordinator) = get_reload_coordinator() {
                    match coordinator.reload(ReloadTarget::Data).await {
                        Ok(result) => {
                            info!("Reload completed in {}ms", result.duration_ms);
                        }
                        Err(e) => {
                            error!("Reload failed: {}", e);
                        }
                    }
                } else {
                    warn!("ReloadCoordinator not initialized, skipping reload");
                }
            }
        });
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

/// Notify server via SIGUSR1
pub fn notify_server() -> Result<()> {
    UnixPlatform::notify_server()
}

/// Setup SIGUSR1 signal handler for reload
///
/// Uses the global ReloadCoordinator to handle reload requests.
pub async fn setup_reload_mechanism() {
    UnixPlatform::setup_reload_mechanism().await
}
