//! MetricsRecorder trait re-exports and Prometheus wrapper.
//!
//! The core `MetricsRecorder` trait and `NoopMetrics` live in
//! `crate::metrics_core` (always compiled).  This module re-exports them
//! for backwards compatibility and adds the Prometheus-specific wrapper.

// Re-export from metrics_core so existing `use crate::metrics::*` keeps working.
pub use crate::metrics_core::{MetricsRecorder, NoopMetrics};

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
