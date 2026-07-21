//! ReloadCoordinator trait and default implementation
//!
//! The ReloadCoordinator provides a unified interface for managing
//! reload operations across the application.

use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::config::try_get_runtime_config;
use crate::errors::Result;
use crate::services::LinkCache;

use super::types::{ReloadResult, ReloadStatus, ReloadTarget};

/// ReloadCoordinator trait
///
/// Defines the interface for managing reload operations.
#[async_trait]
pub trait ReloadCoordinator: Send + Sync {
    /// Execute a reload operation
    async fn reload(&self, target: ReloadTarget) -> Result<ReloadResult>;

    /// Get the current reload status
    fn status(&self) -> ReloadStatus;
}

/// Default implementation of ReloadCoordinator
pub struct DefaultReloadCoordinator {
    cache: Arc<dyn LinkCache + 'static>,
    status: RwLock<ReloadStatus>,
}

impl DefaultReloadCoordinator {
    /// Create a new DefaultReloadCoordinator
    pub fn new(cache: Arc<dyn LinkCache + 'static>) -> Self {
        Self {
            cache,
            status: RwLock::new(ReloadStatus::default()),
        }
    }

    /// Core data reload logic (eliminates code duplication)
    async fn reload_data(&self) -> Result<()> {
        info!("Starting data reload process...");

        // 原子重建所有缓存层（含 Bloom Filter，内部自行从 DB 加载短码）
        self.cache.rebuild_all().await?;

        info!("Data reload process completed successfully");
        Ok(())
    }

    /// Config reload logic
    async fn reload_config(&self) -> Result<()> {
        info!("Starting config reload process...");

        if let Some(rc) = try_get_runtime_config() {
            rc.reload().await?;
            info!("Config reload process completed successfully");
        } else {
            warn!("Runtime config not initialized, skipping config reload");
        }

        Ok(())
    }
}

#[async_trait]
impl ReloadCoordinator for DefaultReloadCoordinator {
    async fn reload(&self, target: ReloadTarget) -> Result<ReloadResult> {
        let started_at = Utc::now();

        // Update status to reloading
        {
            let mut status = self.status.write().await;
            status.is_reloading = true;
            status.current_target = Some(target);
        }

        // Execute reload based on target
        let result = match target {
            ReloadTarget::Data => self.reload_data().await,
            ReloadTarget::Config => self.reload_config().await,
            ReloadTarget::All => {
                let data_result = self.reload_data().await;
                let config_result = self.reload_config().await;
                // Return first error if any
                data_result.and(config_result)
            }
        };

        // Create reload result
        let reload_result = match &result {
            Ok(()) => ReloadResult::success(target, started_at),
            Err(e) => ReloadResult::failure(target, started_at, e.to_string()),
        };

        // Update status
        {
            let mut status = self.status.write().await;
            status.is_reloading = false;
            status.current_target = None;

            match target {
                ReloadTarget::Data => {
                    status.last_data_reload = Some(reload_result.clone());
                }
                ReloadTarget::Config => {
                    status.last_config_reload = Some(reload_result.clone());
                }
                ReloadTarget::All => {
                    status.last_data_reload = Some(reload_result.clone());
                    status.last_config_reload = Some(reload_result.clone());
                }
            }
        }

        if !reload_result.success {
            error!(
                "Reload {} failed: {}",
                target,
                reload_result.message.as_deref().unwrap_or("unknown error")
            );
        }

        result.map(|_| reload_result)
    }

    fn status(&self) -> ReloadStatus {
        // Use try_read to avoid blocking, return default status on failure
        self.status
            .try_read()
            .map(|s| s.clone())
            .unwrap_or_default()
    }
}
