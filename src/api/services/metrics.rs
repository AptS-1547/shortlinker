//! Prometheus metrics endpoint
//!
//! Exposes application metrics in Prometheus text format at `/health/metrics`.

use actix_web::{HttpResponse, Responder};

#[cfg(feature = "metrics")]
use actix_web::web;

#[cfg(feature = "metrics")]
use super::AppStartTime;

#[cfg(feature = "metrics")]
use crate::metrics::METRICS;

/// Metrics service handler
pub struct MetricsService;

impl MetricsService {
    /// Handle metrics export request
    #[cfg(feature = "metrics")]
    pub async fn metrics(app_start_time: web::Data<AppStartTime>) -> impl Responder {
        // Update uptime (cheap operation, fine to do on-demand)
        let now = chrono::Utc::now();
        let uptime = (now - app_start_time.start_datetime).num_seconds().max(0) as f64;
        METRICS.uptime_seconds.set(uptime);

        // System metrics (memory, CPU) are updated by a background task

        // Export metrics in Prometheus format
        match METRICS.export() {
            Ok(output) => HttpResponse::Ok()
                .content_type("text/plain; version=0.0.4; charset=utf-8")
                .body(output),
            Err(e) => {
                tracing::error!("Failed to export metrics: {}", e);
                HttpResponse::InternalServerError()
                    .content_type("text/plain")
                    .body(format!("Failed to export metrics: {}", e))
            }
        }
    }

    /// Metrics not available when feature is disabled
    #[cfg(not(feature = "metrics"))]
    pub async fn metrics() -> impl Responder {
        HttpResponse::NotFound()
            .content_type("text/plain")
            .body("Metrics not enabled. Rebuild with --features metrics")
    }
}
