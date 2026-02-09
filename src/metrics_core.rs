//! Core metrics traits (always compiled, no feature gate).
//!
//! Provides `MetricsRecorder` trait and `NoopMetrics` so that all modules
//! can accept `Arc<dyn MetricsRecorder>` unconditionally.  When the
//! `metrics` feature is disabled, `NoopMetrics` is injected and the
//! compiler optimises every call to a no-op.

use std::sync::Arc;

/// Trait for recording application metrics.
///
/// All methods are no-op by default, allowing partial implementation.
/// Implementations must be thread-safe (Send + Sync).
#[allow(unused_variables)]
pub trait MetricsRecorder: Send + Sync {
    // ===== Analytics (ClickManager) =====

    /// Record click channel dropped event
    fn inc_clicks_channel_dropped(&self, reason: &str) {}

    /// Set current click buffer entries count
    fn set_clicks_buffer_entries(&self, count: f64) {}

    /// Record click flush event
    fn inc_clicks_flush(&self, trigger: &str, status: &str) {}

    // ===== Cache =====

    /// Record cache hit
    fn inc_cache_hit(&self, layer: &str) {}

    /// Record cache miss
    fn inc_cache_miss(&self, layer: &str) {}

    /// Observe cache operation duration
    fn observe_cache_operation(&self, operation: &str, layer: &str, duration_secs: f64) {}

    /// Set cache entry count
    fn set_cache_entries(&self, layer: &str, count: f64) {}

    // ===== Redirect =====

    /// Record Bloom filter false positive
    fn inc_bloom_false_positive(&self) {}

    /// Record redirect response
    fn inc_redirect(&self, status: &str) {}

    // ===== Auth =====

    /// Record authentication failure
    fn inc_auth_failure(&self, method: &str) {}

    // ===== HTTP (timing middleware) =====

    /// Increment active connections counter
    fn inc_active_connections(&self) {}

    /// Decrement active connections counter
    fn dec_active_connections(&self) {}

    /// Observe HTTP request duration
    fn observe_http_request(&self, method: &str, endpoint: &str, status: &str, duration_secs: f64) {
    }

    /// Record HTTP request
    fn inc_http_request(&self, method: &str, endpoint: &str, status: &str) {}

    // ===== Database =====

    /// Observe database query duration
    fn observe_db_query(&self, operation: &str, duration_secs: f64) {}

    /// Record database query
    fn inc_db_query(&self, operation: &str) {}
}

/// Noop metrics implementation for testing and non-metrics builds.
///
/// All methods do nothing, allowing code to run without Prometheus dependencies.
pub struct NoopMetrics;

impl MetricsRecorder for NoopMetrics {}

impl NoopMetrics {
    pub fn new() -> Self {
        Self
    }

    pub fn arc() -> Arc<dyn MetricsRecorder> {
        Arc::new(Self::new())
    }
}

impl Default for NoopMetrics {
    fn default() -> Self {
        Self::new()
    }
}
