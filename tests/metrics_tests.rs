//! Forge-backed metrics integration tests.

#![cfg(feature = "metrics")]

use std::sync::{Arc, OnceLock};
use std::time::Duration;

use actix_web::{App, http::StatusCode, test as actix_test, web};
use aster_forge_metrics::{DbMetricBackend, DbQueryKind, DbQueryMetric};
use shortlinker::metrics::{MetricsRecorder, NoopMetrics};

static METRICS: OnceLock<Arc<dyn MetricsRecorder>> = OnceLock::new();

fn metrics() -> &'static Arc<dyn MetricsRecorder> {
    METRICS.get_or_init(shortlinker::metrics::create_metrics_recorder)
}

#[test]
fn noop_metrics_keeps_product_and_forge_paths_disabled() {
    let noop = NoopMetrics::new();
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

    assert!(!aster_forge_metrics::DbMetricsRecorder::enabled(
        noop.forge_recorder().as_ref()
    ));
}

#[test]
fn forge_and_product_metric_families_are_registered() {
    let forge = metrics().forge_recorder();
    forge.record_http_request("GET", "/health", 200, 0.001);
    aster_forge_metrics::DbMetricsRecorder::record_db_query(
        forge.as_ref(),
        &DbQueryMetric::new(
            DbMetricBackend::Sqlite,
            DbQueryKind::Select,
            false,
            Duration::from_millis(1),
        ),
    );
    let output = aster_forge_metrics::prometheus::export_metrics()
        .expect("Forge metrics export should succeed");

    assert!(output.contains("http_requests_total"));
    assert!(output.contains("db_queries_total"));
    assert!(output.contains("process_uptime_seconds"));
    assert!(output.contains("shortlinker_redirects_total"));
    assert!(output.contains("shortlinker_cache_hits_total"));
    assert!(output.contains("shortlinker_build_info"));
}

#[test]
fn product_cache_metrics_record_with_stable_labels() {
    let metrics = metrics();
    metrics.inc_cache_hit("object_cache");
    metrics.inc_cache_miss("bloom_filter");
    metrics.set_cache_entries("object_cache", 42.0);
    metrics.observe_cache_operation("get", "object_cache", 0.001);

    let output = aster_forge_metrics::prometheus::export_metrics()
        .expect("Forge metrics export should succeed");
    assert!(output.contains("shortlinker_cache_hits_total{layer=\"object_cache\"}"));
    assert!(output.contains("shortlinker_cache_misses_total{layer=\"bloom_filter\"}"));
    assert!(output.contains("shortlinker_cache_entries{layer=\"object_cache\"} 42"));
}

#[test]
fn forge_http_metrics_record_through_shared_recorder() {
    let forge = metrics().forge_recorder();
    forge.record_http_request("GET", "/{code}", 307, 0.005);

    let output = aster_forge_metrics::prometheus::export_metrics()
        .expect("Forge metrics export should succeed");
    assert!(output.contains("http_requests_total"));
    assert!(output.contains("method=\"GET\""));
    assert!(output.contains("route=\"/{code}\""));
    assert!(output.contains("status=\"307\""));
}

#[test]
fn forge_database_metrics_use_backend_kind_and_status() {
    let forge = metrics().forge_recorder();
    aster_forge_metrics::DbMetricsRecorder::record_db_query(
        forge.as_ref(),
        &DbQueryMetric::new(
            DbMetricBackend::Sqlite,
            DbQueryKind::Select,
            false,
            Duration::from_millis(2),
        ),
    );

    let output = aster_forge_metrics::prometheus::export_metrics()
        .expect("Forge metrics export should succeed");
    assert!(output.contains("db_queries_total{backend=\"sqlite\",kind=\"select\",status=\"ok\"}"));
}

#[test]
fn redirect_auth_and_bloom_metrics_remain_product_owned() {
    let metrics = metrics();
    metrics.inc_redirect("307");
    metrics.inc_redirect("404");
    metrics.inc_auth_failure("bearer");
    metrics.inc_bloom_false_positive();

    let output = aster_forge_metrics::prometheus::export_metrics()
        .expect("Forge metrics export should succeed");
    assert!(output.contains("shortlinker_redirects_total{status=\"307\"}"));
    assert!(output.contains("shortlinker_redirects_total{status=\"404\"}"));
    assert!(output.contains("shortlinker_auth_failures_total{method=\"bearer\"}"));
    assert!(output.contains("shortlinker_bloom_filter_false_positives_total"));
}

#[test]
fn click_buffer_metrics_remain_product_owned() {
    let metrics = metrics();
    metrics.set_clicks_buffer_entries(10.0);
    metrics.inc_clicks_flush("interval", "success");
    metrics.inc_clicks_channel_dropped("full");

    let output = aster_forge_metrics::prometheus::export_metrics()
        .expect("Forge metrics export should succeed");
    assert!(output.contains("shortlinker_clicks_buffer_entries 10"));
    assert!(output.contains("shortlinker_clicks_flush_total"));
    assert!(output.contains("trigger=\"interval\""));
    assert!(output.contains("status=\"success\""));
    assert!(output.contains("shortlinker_clicks_channel_dropped_total{reason=\"full\"}"));
}

#[test]
fn build_info_uses_package_version() {
    let _ = metrics();
    let output = aster_forge_metrics::prometheus::export_metrics()
        .expect("Forge metrics export should succeed");
    assert!(output.contains(&format!(
        "shortlinker_build_info{{version=\"{}\"}} 1",
        env!("CARGO_PKG_VERSION")
    )));
}

#[actix_web::test]
async fn health_scope_mounts_forge_prometheus_endpoint() {
    let _ = metrics();
    let app = actix_test::init_service(App::new().service(
        web::scope("/health").service(shortlinker::api::services::health::health_routes()),
    ))
    .await;

    let response = actix_test::call_service(
        &app,
        actix_test::TestRequest::get()
            .uri("/health/metrics")
            .to_request(),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = actix_test::read_body(response).await;
    assert!(
        std::str::from_utf8(&body)
            .expect("metrics response should be UTF-8")
            .contains("http_requests_total")
    );
}
