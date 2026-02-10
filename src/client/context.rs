//! Lazy-initialized service context for fallback mode
//!
//! Storage and services are only created when the IPC fallback path is actually needed.

use std::sync::Arc;
use tokio::sync::OnceCell;

use crate::cache::NullCompositeCache;
use crate::errors::ShortlinkerError;
use crate::metrics_core::NoopMetrics;
use crate::services::{ConfigService, LinkService};
use crate::storage::{SeaOrmStorage, StorageFactory};

use super::ClientError;

/// Lazy-initialized service context for CLI/TUI fallback mode.
///
/// Created once per CLI invocation or TUI session.
/// Storage and services are only initialized when first needed (i.e., when
/// the server is not running and we fall back to local operations).
pub struct ServiceContext {
    storage: OnceCell<Arc<SeaOrmStorage>>,
    link_service: OnceCell<Arc<LinkService>>,
    config_service: OnceCell<Arc<ConfigService>>,
}

impl Default for ServiceContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceContext {
    /// Create an empty context (CLI usage — storage created lazily)
    pub fn new() -> Self {
        Self {
            storage: OnceCell::new(),
            link_service: OnceCell::new(),
            config_service: OnceCell::new(),
        }
    }

    /// Create a context with pre-injected storage (TUI usage — already has storage)
    pub fn with_storage(storage: Arc<SeaOrmStorage>) -> Self {
        let ctx = Self::new();
        let _ = ctx.storage.set(storage);
        ctx
    }

    /// Get or lazily create storage
    async fn get_storage(&self) -> Result<&Arc<SeaOrmStorage>, ClientError> {
        self.storage
            .get_or_try_init(|| async {
                StorageFactory::create(NoopMetrics::arc())
                    .await
                    .map_err(|e| ClientError::InitFailed(format!("Storage init failed: {}", e)))
            })
            .await
    }

    /// Get or lazily create LinkService
    pub async fn get_link_service(&self) -> Result<&Arc<LinkService>, ClientError> {
        // Ensure storage is initialized first
        let storage = self.get_storage().await?.clone();
        self.link_service
            .get_or_try_init(|| async {
                let cache = NullCompositeCache::arc();
                Ok(Arc::new(LinkService::new(storage, cache)))
            })
            .await
    }

    /// Get or lazily create ConfigService
    pub async fn get_config_service(&self) -> Result<&Arc<ConfigService>, ClientError> {
        self.config_service
            .get_or_try_init(|| async {
                ConfigService::new()
                    .map(Arc::new)
                    .map_err(|e: ShortlinkerError| {
                        ClientError::InitFailed(format!("ConfigService init failed: {}", e))
                    })
            })
            .await
    }
}
