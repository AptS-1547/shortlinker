//! Reload operations module
//!
//! This module provides:
//! - `reload_all`: Manual cache/storage reload function (used by admin API)
//! - `setup_reload_mechanism`: **DEPRECATED** - Use `system::platform::setup_reload_mechanism` instead
//!
//! The automatic reload mechanism has been moved to the platform abstraction layer.

use crate::cache::{CompositeCacheTrait, traits::BloomConfig};
use crate::storage::SeaOrmStorage;
use std::sync::Arc;

/// Manually reload cache and storage
///
/// This function is used by the admin API to reload data after modifications.
/// For automatic reload mechanisms (signals/polling), use `system::platform::setup_reload_mechanism`.
pub async fn reload_all(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<SeaOrmStorage>,
) -> anyhow::Result<()> {
    tracing::info!("Starting reload process...");

    // 重新加载存储
    storage.reload().await?;
    let links = storage.load_all().await?;

    // 重新配置缓存
    cache
        .reconfigure(BloomConfig {
            capacity: links.len(),
            fp_rate: 0.001,
        })
        .await?;

    // 加载缓存
    cache.load_cache(links).await;

    tracing::info!("Reload process completed successfully");
    Ok(())
}
