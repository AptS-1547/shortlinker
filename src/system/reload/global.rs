//! Global ReloadCoordinator instance management
//!
//! This module provides global access to the ReloadCoordinator instance.

use std::sync::{Arc, OnceLock};

use super::coordinator::ReloadCoordinator;

/// Global ReloadCoordinator instance
static RELOAD_COORDINATOR: OnceLock<Arc<dyn ReloadCoordinator>> = OnceLock::new();

/// Initialize the global ReloadCoordinator
///
/// This should be called once during application startup,
/// after cache and storage are initialized.
pub fn init_reload_coordinator(coordinator: Arc<dyn ReloadCoordinator>) {
    let _ = RELOAD_COORDINATOR.set(coordinator);
}

/// Get the global ReloadCoordinator
///
/// Returns None if the coordinator has not been initialized.
pub fn get_reload_coordinator() -> Option<Arc<dyn ReloadCoordinator>> {
    RELOAD_COORDINATOR.get().cloned()
}
