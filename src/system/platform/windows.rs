//! Windows platform implementation
//!
//! This module provides simplified Windows platform support:
//! - Lock file-based instance management (no process checking)
//! - File-based notification mechanism (trigger files)
//! - Polling-based reload mechanism (checks every 3 seconds)
//!
//! Note: Windows implementation is simplified compared to Unix due to
//! lack of signal support. It uses file-based polling instead.

use crate::cache::{CompositeCacheTrait, traits::BloomConfig};
use crate::errors::{Result, ShortlinkerError};
use crate::storage::SeaOrmStorage;
use std::fs;
use std::sync::Arc;
use tracing::{error, info, warn};

use super::PlatformOps;

/// Windows platform operations implementation
pub struct WindowsPlatform;

impl PlatformOps for WindowsPlatform {
    fn init_lockfile() -> std::io::Result<()> {
        use std::io::{self, Write};
        use std::path::Path;

        let lock_file = ".shortlinker.lock";

        // Check if lock file already exists
        if Path::new(lock_file).exists() {
            error!("Server already running, stop it first");
            error!(
                "If the server is not running, delete the lock file: {}",
                lock_file
            );
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Server is already running",
            ));
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
    }

    fn notify_server() -> Result<()> {
        // On Windows, use a trigger file
        match fs::write("shortlinker.reload", "") {
            Ok(_) => Ok(()),
            Err(e) => Err(ShortlinkerError::notify_server(format!(
                "Failed to notify server: {}",
                e
            ))),
        }
    }

    async fn setup_reload_mechanism(
        cache: Arc<dyn CompositeCacheTrait + 'static>,
        storage: Arc<SeaOrmStorage>,
    ) {
        use std::time::SystemTime;
        use tokio::fs;
        use tokio::time::{Duration, sleep};

        let reload_file = "shortlinker.reload";
        let mut last_check = SystemTime::now();
        let check_interval = Duration::from_secs(3);

        tokio::spawn(async move {
            loop {
                // Sleep for the check interval
                sleep(check_interval).await;

                // Check if reload file exists and was modified
                match fs::metadata(reload_file).await {
                    Ok(metadata) => {
                        match metadata.modified() {
                            Ok(modified) => {
                                if modified > last_check {
                                    info!("Reload request detected, reloading...");

                                    match reload_all(cache.clone(), storage.clone()).await {
                                        Ok(_) => {
                                            info!("Reload successful");
                                            last_check = SystemTime::now();

                                            // Remove the trigger file
                                            if let Err(e) = fs::remove_file(reload_file).await {
                                                warn!("Failed to remove reload file: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            error!("Reload failed: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to get file modification time: {}", e);
                            }
                        }
                    }
                    Err(_) => {
                        // File doesn't exist, this is normal
                    }
                }
            }
        });
    }
}

/// Reload cache and storage
///
/// This function is called when a reload trigger is detected.
/// It reloads the storage backend and rebuilds the cache.
async fn reload_all(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<SeaOrmStorage>,
) -> anyhow::Result<()> {
    info!("Starting reload process...");

    // Reload storage backend
    storage.reload().await?;
    let links = storage.load_all().await?;

    // Reconfigure cache with new capacity
    cache
        .reconfigure(BloomConfig {
            capacity: links.len(),
            fp_rate: 0.001,
        })
        .await?;

    // Load data into cache
    cache.load_cache(links).await;

    info!("Reload process completed successfully");
    Ok(())
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

/// Notify server via trigger file
pub fn notify_server() -> Result<()> {
    WindowsPlatform::notify_server()
}

/// Setup file polling for reload
pub async fn setup_reload_mechanism(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<SeaOrmStorage>,
) {
    WindowsPlatform::setup_reload_mechanism(cache, storage).await
}
