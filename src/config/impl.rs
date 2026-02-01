use std::sync::{Arc, OnceLock};

use arc_swap::ArcSwap;

use super::StaticConfig;

static CONFIG: OnceLock<ArcSwap<StaticConfig>> = OnceLock::new();

/// Get the global configuration instance
///
/// Returns an Arc pointer to the configuration, which is cheap to clone
/// and doesn't hold any locks.
pub fn get_config() -> Arc<StaticConfig> {
    CONFIG
        .get()
        .expect("Config not initialized. Call init_config() first.")
        .load_full()
}

/// Initialize the global configuration
///
/// Loads configuration from "config.toml" in the current directory.
/// If the file doesn't exist, uses in-memory defaults.
///
/// # Examples
/// ```no_run
/// use shortlinker::config::init_config;
/// init_config();
/// ```
pub fn init_config() {
    CONFIG.get_or_init(|| ArcSwap::from_pointee(StaticConfig::load()));
}
