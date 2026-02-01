//! Reload type definitions
//!
//! This module defines the types used for the reload system:
//! - `ReloadTarget`: What to reload (data, config, or all)
//! - `ReloadResult`: Result of a reload operation
//! - `ReloadEvent`: Events emitted during reload
//! - `ReloadStatus`: Current reload system status

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Reload target type
///
/// Specifies what should be reloaded when triggering a reload operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReloadTarget {
    /// Data reload: Storage + Bloom Filter + Cache
    ///
    /// Triggered by:
    /// - SIGUSR1 signal (Unix)
    /// - shortlinker.reload file modification (Windows)
    /// - CLI link management commands
    Data,

    /// Config reload: RuntimeConfig (database -> cache)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reload_target_display() {
        assert_eq!(format!("{}", ReloadTarget::Data), "data");
        assert_eq!(format!("{}", ReloadTarget::Config), "config");
        assert_eq!(format!("{}", ReloadTarget::All), "all");
    }

    #[test]
    fn test_reload_target_equality() {
        assert_eq!(ReloadTarget::Data, ReloadTarget::Data);
        assert_ne!(ReloadTarget::Data, ReloadTarget::Config);
        assert_ne!(ReloadTarget::Config, ReloadTarget::All);
    }

    #[test]
    fn test_reload_target_serialization() {
        let target = ReloadTarget::Data;
        let json = serde_json::to_string(&target).unwrap();
        let decoded: ReloadTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, ReloadTarget::Data);

        let target = ReloadTarget::All;
        let json = serde_json::to_string(&target).unwrap();
        let decoded: ReloadTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, ReloadTarget::All);
    }

    #[test]
    fn test_reload_result_success() {
        let started = Utc::now();
        std::thread::sleep(std::time::Duration::from_millis(10));
        let result = ReloadResult::success(ReloadTarget::Data, started);

        assert!(result.success);
        assert_eq!(result.target, ReloadTarget::Data);
        assert!(result.message.is_none());
        assert!(result.duration_ms >= 10);
        assert!(result.finished_at >= result.started_at);
    }

    #[test]
    fn test_reload_result_failure() {
        let started = Utc::now();
        let result = ReloadResult::failure(
            ReloadTarget::Config,
            started,
            "Database connection failed".to_string(),
        );

        assert!(!result.success);
        assert_eq!(result.target, ReloadTarget::Config);
        assert_eq!(
            result.message,
            Some("Database connection failed".to_string())
        );
    }

    #[test]
    fn test_reload_result_duration_non_negative() {
        // duration_ms 是 u64，始终非负
        let started = Utc::now();
        let result = ReloadResult::success(ReloadTarget::Data, started);
        // u64 类型保证非负，这里只验证结果存在
        let _ = result.duration_ms;
    }

    #[test]
    fn test_reload_event_variants() {
        let event = ReloadEvent::Started {
            target: ReloadTarget::Data,
        };
        if let ReloadEvent::Started { target } = event {
            assert_eq!(target, ReloadTarget::Data);
        }

        let result = ReloadResult::success(ReloadTarget::Config, Utc::now());
        let event = ReloadEvent::Completed {
            result: result.clone(),
        };
        if let ReloadEvent::Completed { result: r } = event {
            assert!(r.success);
        }

        let event = ReloadEvent::Failed {
            target: ReloadTarget::All,
            error: "test error".to_string(),
        };
        if let ReloadEvent::Failed { target, error } = event {
            assert_eq!(target, ReloadTarget::All);
            assert_eq!(error, "test error");
        }
    }

    #[test]
    fn test_reload_status_default() {
        let status = ReloadStatus::default();
        assert!(status.last_data_reload.is_none());
        assert!(status.last_config_reload.is_none());
        assert!(!status.is_reloading);
        assert!(status.current_target.is_none());
    }

    #[test]
    fn test_reload_status_with_results() {
        let mut status = ReloadStatus::default();

        let data_result = ReloadResult::success(ReloadTarget::Data, Utc::now());
        status.last_data_reload = Some(data_result);

        let config_result =
            ReloadResult::failure(ReloadTarget::Config, Utc::now(), "error".to_string());
        status.last_config_reload = Some(config_result);

        assert!(status.last_data_reload.as_ref().unwrap().success);
        assert!(!status.last_config_reload.as_ref().unwrap().success);
    }

    #[test]
    fn test_reload_target_clone_and_copy() {
        let target = ReloadTarget::Data;
        let cloned = target;
        let copied = target; // Copy trait
        assert_eq!(target, cloned);
        assert_eq!(target, copied);
    }

    #[test]
    fn test_reload_target_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(ReloadTarget::Data);
        set.insert(ReloadTarget::Config);
        set.insert(ReloadTarget::All);
        assert_eq!(set.len(), 3);

        // 重复插入不会增加数量
        set.insert(ReloadTarget::Data);
        assert_eq!(set.len(), 3);
    }
}
