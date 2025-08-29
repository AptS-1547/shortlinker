pub mod app_config;
pub mod lifetime;
mod lockfile;
pub mod reload;
#[cfg(any(feature = "cli", feature = "tui"))]
pub mod signal;

#[cfg(any(feature = "cli", feature = "tui"))]
pub use reload::setup_reload_mechanism;
#[cfg(any(feature = "cli", feature = "tui"))]
pub use signal::notify_server;
