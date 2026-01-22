//! Reload operations module
//!
//! This module provides a unified reload system with:
//! - `ReloadCoordinator`: Trait for managing reload operations
//! - `ReloadTarget`: What to reload (Data, Config, or All)
//! - `ReloadEvent`: Events emitted during reload
//! - Global instance management
//!
//! # Architecture
//!
//! The reload system is divided into two types of reloads:
//! - **Data reload**: Reloads storage and cache (Bloom filter, object cache)
//! - **Config reload**: Reloads runtime configuration from database
//!
//! # Usage
//!
//! ```ignore
//! use crate::system::reload::{get_reload_coordinator, ReloadTarget};
//!
//! // Get the coordinator
//! if let Some(coordinator) = get_reload_coordinator() {
//!     // Trigger a data reload
//!     coordinator.reload(ReloadTarget::Data).await?;
//!
//!     // Or reload everything
//!     coordinator.reload(ReloadTarget::All).await?;
//! }
//! ```

mod coordinator;
mod global;
mod types;

pub use coordinator::{DefaultReloadCoordinator, ReloadCoordinator};
pub use global::{get_reload_coordinator, init_default_coordinator, init_reload_coordinator};
pub use types::{ReloadEvent, ReloadResult, ReloadStatus, ReloadTarget};

// Backward compatibility: keep the old reload_all function signature
// but mark it as deprecated
use crate::cache::CompositeCacheTrait;
use crate::storage::SeaOrmStorage;
use std::sync::Arc;

/// Manually reload cache and storage
///
/// This function is deprecated. Use `ReloadCoordinator::reload(ReloadTarget::Data)` instead.
///
/// This function is kept for backward compatibility with existing code.
#[deprecated(
    since = "0.5.0",
    note = "Use ReloadCoordinator::reload(ReloadTarget::Data) instead"
)]
pub async fn reload_all(
    cache: Arc<dyn CompositeCacheTrait + 'static>,
    storage: Arc<SeaOrmStorage>,
) -> anyhow::Result<()> {
    // Create a temporary coordinator to execute reload
    let coordinator = DefaultReloadCoordinator::new(cache, storage);
    coordinator.reload(ReloadTarget::Data).await?;
    Ok(())
}
