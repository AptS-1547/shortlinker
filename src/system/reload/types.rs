//! Reload type definitions
//!
//! This module defines the types used for the reload system:
//! - `ReloadTarget`: What to reload (data, config, or all)
//! - `ReloadResult`: Result of a reload operation
//! - `ReloadEvent`: Events emitted during reload
//! - `ReloadStatus`: Current reload system status

use chrono::{DateTime, Utc};

/// Reload target type
///
/// Specifies what should be reloaded when triggering a reload operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReloadTarget {
    /// Data reload: Storage + Bloom Filter + Cache
    ///
    /// Triggered by:
    /// - SIGUSR1 signal (Unix)
    /// - shortlinker.reload file modification (Windows)
    /// - CLI link management commands
    Data,

    /// Config reload: RuntimeConfig (database -> AppConfig)
    ///
    /// Triggered by:
    /// - HTTP POST /admin/v1/config/reload
    /// - CLI config management commands
    Config,

    /// Reload both data and config
    All,
}

impl std::fmt::Display for ReloadTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReloadTarget::Data => write!(f, "data"),
            ReloadTarget::Config => write!(f, "config"),
            ReloadTarget::All => write!(f, "all"),
        }
    }
}

/// Result of a reload operation
#[derive(Debug, Clone)]
pub struct ReloadResult {
    /// What was reloaded
    pub target: ReloadTarget,
    /// Whether the reload was successful
    pub success: bool,
    /// Error message if failed
    pub message: Option<String>,
    /// When the reload started
    pub started_at: DateTime<Utc>,
    /// When the reload finished
    pub finished_at: DateTime<Utc>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl ReloadResult {
    /// Create a successful reload result
    pub fn success(target: ReloadTarget, started_at: DateTime<Utc>) -> Self {
        let finished_at = Utc::now();
        let duration_ms = (finished_at - started_at).num_milliseconds().max(0) as u64;
        Self {
            target,
            success: true,
            message: None,
            started_at,
            finished_at,
            duration_ms,
        }
    }

    /// Create a failed reload result
    pub fn failure(target: ReloadTarget, started_at: DateTime<Utc>, error: String) -> Self {
        let finished_at = Utc::now();
        let duration_ms = (finished_at - started_at).num_milliseconds().max(0) as u64;
        Self {
            target,
            success: false,
            message: Some(error),
            started_at,
            finished_at,
            duration_ms,
        }
    }
}

/// Events emitted during reload operations
///
/// These events can be subscribed to for monitoring reload progress.
#[derive(Debug, Clone)]
pub enum ReloadEvent {
    /// Reload operation started
    Started { target: ReloadTarget },
    /// Reload operation completed successfully
    Completed { result: ReloadResult },
    /// Reload operation failed
    Failed { target: ReloadTarget, error: String },
}

/// Current status of the reload system
#[derive(Debug, Clone, Default)]
pub struct ReloadStatus {
    /// Last data reload result
    pub last_data_reload: Option<ReloadResult>,
    /// Last config reload result
    pub last_config_reload: Option<ReloadResult>,
    /// Whether a reload is currently in progress
    pub is_reloading: bool,
    /// Current reload target (if reloading)
    pub current_target: Option<ReloadTarget>,
}
