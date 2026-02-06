//! MetricsRecorder trait for dependency injection
//!
//! This trait abstracts the metrics recording interface, allowing:
//! - Production use with Prometheus metrics
//! - Testing with NoopMetrics or custom mock implementations
//! - Better testability and module decoupling

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

/// Noop metrics implementation for testing.
///
/// All methods do nothing, allowing tests to run without Prometheus dependencies.
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

/// Wrapper that delegates to the global METRICS singleton.
///
/// This allows the global `Lazy<Metrics>` to be used with `Arc<dyn MetricsRecorder>`
/// since `Lazy<T>` doesn't implement `Clone`.
pub struct PrometheusMetricsWrapper;

impl MetricsRecorder for PrometheusMetricsWrapper {
    fn inc_clicks_channel_dropped(&self, reason: &str) {
        super::METRICS.inc_clicks_channel_dropped(reason);
    }

    fn set_clicks_buffer_entries(&self, count: f64) {
        super::METRICS.set_clicks_buffer_entries(count);
    }

    fn inc_clicks_flush(&self, trigger: &str, status: &str) {
        super::METRICS.inc_clicks_flush(trigger, status);
    }

    fn inc_cache_hit(&self, layer: &str) {
        super::METRICS.inc_cache_hit(layer);
    }

    fn inc_cache_miss(&self, layer: &str) {
        super::METRICS.inc_cache_miss(layer);
    }

    fn observe_cache_operation(&self, operation: &str, layer: &str, duration_secs: f64) {
        super::METRICS.observe_cache_operation(operation, layer, duration_secs);
    }

    fn set_cache_entries(&self, layer: &str, count: f64) {
        super::METRICS.set_cache_entries(layer, count);
    }

    fn inc_bloom_false_positive(&self) {
        super::METRICS.inc_bloom_false_positive();
    }

    fn inc_redirect(&self, status: &str) {
        super::METRICS.inc_redirect(status);
    }

    fn inc_auth_failure(&self, method: &str) {
        super::METRICS.inc_auth_failure(method);
    }

    fn inc_active_connections(&self) {
        super::METRICS.inc_active_connections();
    }

    fn dec_active_connections(&self) {
        super::METRICS.dec_active_connections();
    }

    fn observe_http_request(&self, method: &str, endpoint: &str, status: &str, duration_secs: f64) {
        super::METRICS.observe_http_request(method, endpoint, status, duration_secs);
    }

    fn inc_http_request(&self, method: &str, endpoint: &str, status: &str) {
        super::METRICS.inc_http_request(method, endpoint, status);
    }

    fn observe_db_query(&self, operation: &str, duration_secs: f64) {
        super::METRICS.observe_db_query(operation, duration_secs);
    }

    fn inc_db_query(&self, operation: &str) {
        super::METRICS.inc_db_query(operation);
    }
}
