//! Prometheus metrics endpoint
//!
//! Exposes application metrics in Prometheus text format at `/health/metrics`.

use actix_web::{HttpResponse, Responder};

#[cfg(feature = "metrics")]
use actix_web::web;

#[cfg(feature = "metrics")]
use super::AppStartTime;

#[cfg(feature = "metrics")]
use crate::metrics::{METRICS, update_system_metrics};

/// Metrics service handler
pub struct MetricsService;

impl MetricsService {
    /// Handle metrics export request
    #[cfg(feature = "metrics")]
    pub async fn metrics(app_start_time: web::Data<AppStartTime>) -> impl Responder {
        // Update uptime
        let now = chrono::Utc::now();
        let uptime = (now - app_start_time.start_datetime).num_seconds().max(0) as f64;
        METRICS.uptime_seconds.set(uptime);

        // Update system metrics (memory, CPU)
        update_system_metrics();

        // Export metrics in Prometheus format
        let output = METRICS.export();

        HttpResponse::Ok()
            .content_type("text/plain; version=0.0.4; charset=utf-8")
            .body(output)
    }

    /// Metrics not available when feature is disabled
    #[cfg(not(feature = "metrics"))]
    pub async fn metrics() -> impl Responder {
        HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Metrics not enabled. Rebuild with --features metrics")
    }
}
