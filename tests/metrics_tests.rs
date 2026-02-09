//! Metrics module tests
//!
//! Tests for the metrics recording, export, and trait implementations.
//! These tests require the `metrics` feature to be enabled:
//!   cargo test --features metrics --test metrics_tests

// Only compile when the metrics feature is enabled
#![cfg(feature = "metrics")]

use shortlinker::config::init_config;
use shortlinker::metrics::init_metrics;
use shortlinker::metrics_core::{MetricsRecorder, NoopMetrics};

use std::sync::Once;

static INIT: Once = Once::new();

fn init_test_env() {
    INIT.call_once(|| {
        init_config();
        init_metrics().expect("Failed to init metrics");
    });
}

// =============================================================================
// NoopMetrics Tests (always available, but grouped here)
// =============================================================================

#[test]
fn test_noop_metrics_implements_trait() {
    let noop = NoopMetrics::new();
    // All methods should be callable without panic
    noop.inc_clicks_channel_dropped("test");
    noop.set_clicks_buffer_entries(42.0);
    noop.inc_clicks_flush("interval", "success");
    noop.inc_cache_hit("object_cache");
    noop.inc_cache_miss("bloom_filter");
    noop.observe_cache_operation("get", "object_cache", 0.001);
    noop.set_cache_entries("object_cache", 100.0);
    noop.inc_bloom_false_positive();
    noop.inc_redirect("307");
    noop.inc_auth_failure("bearer");
    noop.inc_active_connections();
    noop.dec_active_connections();
    noop.observe_http_request("GET", "redirect", "200", 0.01);
    noop.inc_http_request("POST", "admin", "201");
    noop.observe_db_query("select", 0.005);
    noop.inc_db_query("insert");
}

#[test]
fn test_noop_metrics_arc() {
    let arc = NoopMetrics::arc();
    arc.inc_redirect("307");
    arc.inc_cache_hit("bloom_filter");
}

#[test]
fn test_noop_metrics_default() {
    let noop = NoopMetrics;
    noop.inc_redirect("404");
}

// =============================================================================
// Prometheus Metrics Tests (requires `metrics` feature)
// =============================================================================

#[test]
fn test_init_metrics_succeeds() {
    init_test_env();
    let metrics = shortlinker::metrics::get_metrics();
    assert!(metrics.is_some(), "Metrics should be initialized");
}

#[test]
fn test_init_metrics_idempotent() {
    init_test_env();
    // Calling init_metrics again should not error
    let result = init_metrics();
    assert!(result.is_ok());
}

#[test]
fn test_metrics_export_contains_expected_metrics() {
    init_test_env();
    let metrics = shortlinker::metrics::get_metrics().unwrap();

    let output = metrics.export().expect("Export should succeed");

    // Verify key metric names appear in the Prometheus text output
    assert!(
        output.contains("shortlinker_http_requests_total"),
        "Missing http_requests_total"
    );
    assert!(
        output.contains("shortlinker_redirects_total"),
        "Missing redirects_total"
    );
    assert!(
        output.contains("shortlinker_cache_hits_total"),
        "Missing cache_hits_total"
    );
    assert!(
        output.contains("shortlinker_db_queries_total"),
        "Missing db_queries_total"
    );
    assert!(
        output.contains("shortlinker_build_info"),
        "Missing build_info"
    );
    assert!(
        output.contains("shortlinker_uptime_seconds"),
        "Missing uptime_seconds"
    );
}

#[test]
fn test_metrics_recording_cache() {
    init_test_env();
    let metrics = shortlinker::metrics::get_metrics().unwrap();

    // Record some cache metrics
    metrics.inc_cache_hit("object_cache");
    metrics.inc_cache_hit("object_cache");
    metrics.inc_cache_miss("bloom_filter");
    metrics.set_cache_entries("object_cache", 42.0);
    metrics.observe_cache_operation("get", "object_cache", 0.001);

    let output = metrics.export().unwrap();
    // Verify the cache hit counter was incremented
    assert!(output.contains("shortlinker_cache_hits_total{layer=\"object_cache\"}"));
    assert!(output.contains("shortlinker_cache_misses_total{layer=\"bloom_filter\"}"));
    assert!(output.contains("shortlinker_cache_entries{layer=\"object_cache\"} 42"));
}

#[test]
fn test_metrics_recording_http() {
    init_test_env();
    let metrics = shortlinker::metrics::get_metrics().unwrap();

    metrics.inc_http_request("GET", "redirect", "307");
    metrics.observe_http_request("GET", "redirect", "307", 0.005);
    metrics.inc_active_connections();
    metrics.dec_active_connections();

    let output = metrics.export().unwrap();
    // Check for metric name and labels separately (Prometheus format may vary)
    assert!(output.contains("shortlinker_http_requests_total"));
    assert!(output.contains("method=\"GET\""));
    assert!(output.contains("endpoint=\"redirect\""));
    assert!(output.contains("status=\"307\""));
}

#[test]
fn test_metrics_recording_db() {
    init_test_env();
    let metrics = shortlinker::metrics::get_metrics().unwrap();

    metrics.inc_db_query("select");
    metrics.observe_db_query("select", 0.002);

    let output = metrics.export().unwrap();
    assert!(output.contains("shortlinker_db_queries_total{operation=\"select\"}"));
}

#[test]
fn test_metrics_recording_redirect() {
    init_test_env();
    let metrics = shortlinker::metrics::get_metrics().unwrap();

    metrics.inc_redirect("307");
    metrics.inc_redirect("404");
    metrics.inc_bloom_false_positive();

    let output = metrics.export().unwrap();
    assert!(output.contains("shortlinker_redirects_total{status=\"307\"}"));
    assert!(output.contains("shortlinker_redirects_total{status=\"404\"}"));
    assert!(output.contains("shortlinker_bloom_filter_false_positives_total"));
}

#[test]
fn test_metrics_recording_auth() {
    init_test_env();
    let metrics = shortlinker::metrics::get_metrics().unwrap();

    metrics.inc_auth_failure("bearer");
    metrics.inc_auth_failure("cookie");

    let output = metrics.export().unwrap();
    assert!(output.contains("shortlinker_auth_failures_total{method=\"bearer\"}"));
    assert!(output.contains("shortlinker_auth_failures_total{method=\"cookie\"}"));
}

#[test]
fn test_metrics_recording_clicks() {
    init_test_env();
    let metrics = shortlinker::metrics::get_metrics().unwrap();

    metrics.set_clicks_buffer_entries(10.0);
    metrics.inc_clicks_flush("interval", "success");
    metrics.inc_clicks_channel_dropped("full");

    let output = metrics.export().unwrap();
    assert!(output.contains("shortlinker_clicks_buffer_entries 10"));
    // Check for metric name and labels separately
    assert!(output.contains("shortlinker_clicks_flush_total"));
    assert!(output.contains("trigger=\"interval\""));
    assert!(output.contains("status=\"success\""));
    assert!(output.contains("shortlinker_clicks_channel_dropped"));
    assert!(output.contains("reason=\"full\""));
}

#[test]
fn test_metrics_build_info() {
    init_test_env();
    let metrics = shortlinker::metrics::get_metrics().unwrap();

    let output = metrics.export().unwrap();
    // build_info should have version label set to 1.0
    assert!(output.contains("shortlinker_build_info{version="));
    assert!(output.contains("} 1"));
}

#[test]
fn test_prometheus_wrapper_delegates() {
    init_test_env();
    let wrapper = shortlinker::metrics::PrometheusMetricsWrapper;

    // All calls should delegate to the global metrics without panic
    wrapper.inc_redirect("307");
    wrapper.inc_cache_hit("object_cache");
    wrapper.inc_http_request("GET", "redirect", "200");
    wrapper.inc_db_query("select");
    wrapper.inc_auth_failure("bearer");
    wrapper.inc_bloom_false_positive();
    wrapper.inc_active_connections();
    wrapper.dec_active_connections();
}
