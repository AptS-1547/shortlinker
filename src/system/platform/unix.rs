//! Unix/Linux platform implementation
//!
//! This module provides full-featured Unix/Linux platform support:
//! - PID file management with process checking
//! - SIGUSR1 signal-based reload mechanism
//! - Efficient signal-driven notification

use crate::cache::{CompositeCacheTrait, traits::BloomConfig};
use crate::errors::{Result, ShortlinkerError};
use crate::repository::Repository;
use std::fs;
use std::sync::Arc;
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

        // Check if PID file already exists
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
                            // Process is still running
                            error!("Server already running (PID: {}), stop it first", old_pid);
                            error!("You can stop it with:");
                            error!("  kill {}", old_pid);
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

    async fn setup_reload_mechanism(
        cache: Arc<dyn CompositeCacheTrait + 'static>,
        repository: Arc<dyn Repository + 'static>,
    ) {
        use tokio::signal::unix::{SignalKind, signal};

        tokio::spawn(async move {
            let mut stream =
                signal(SignalKind::user_defined1()).expect("Failed to create SIGUSR1 handler");

            while (stream.recv().await).is_some() {
                info!("Received SIGUSR1, reloading...");

                if let Err(e) = reload_all(cache.clone(), repository.clone()).await {
                    error!("Reload failed: {}", e);
                } else {
                    info!("Reload successful");
                }
            }
        });
    }
}

/// Reload cache and repository
///
/// This function is called when a reload signal is received.
/// It reloads the repository backend and rebuilds the cache.
async fn reload_all(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    repository: Arc<dyn Repository + 'static>,
) -> anyhow::Result<()> {
    info!("Starting reload process...");

    // Reload repository backend
    repository.reload().await?;
    let links = repository.load_all().await;

    // Reconfigure cache with new capacity
    cache
        .reconfigure(BloomConfig {
            capacity: links.len(),
            fp_rate: 0.001,
        })
        .await;

    // Load data into cache
    cache.load_cache(links).await;

    info!("Reload process completed successfully");
    Ok(())
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
pub async fn setup_reload_mechanism(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    repository: Arc<dyn Repository + 'static>,
) {
    UnixPlatform::setup_reload_mechanism(cache, repository).await
}
