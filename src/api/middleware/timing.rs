//! HTTP timing middleware
//!
//! Records HTTP request duration, request count, and active connections.
//! Only active when the `metrics` feature is enabled.

use actix_service::{Service, Transform};
use actix_web::{
    Error,
    dev::{ServiceRequest, ServiceResponse},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use std::time::Instant;

#[cfg(feature = "metrics")]
use crate::metrics::METRICS;

/// HTTP timing middleware factory
#[derive(Clone, Default)]
pub struct TimingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for TimingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TimingService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TimingService {
            service: Rc::new(service),
        }))
    }
}

pub struct TimingService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for TimingService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let start = Instant::now();

        // Extract method and endpoint for labels
        #[cfg(feature = "metrics")]
        let method = req.method().to_string();
        #[cfg(feature = "metrics")]
        let endpoint = classify_endpoint(req.path());

        // Increment active connections
        #[cfg(feature = "metrics")]
        METRICS.http_active_connections.inc();

        Box::pin(async move {
            let result = srv.call(req).await;

            // Decrement active connections
            #[cfg(feature = "metrics")]
            METRICS.http_active_connections.dec();

            #[cfg(feature = "metrics")]
            {
                let duration = start.elapsed().as_secs_f64();
                let status = match &result {
                    Ok(response) => response.status().as_u16().to_string(),
                    Err(_) => "500".to_string(),
                };

                METRICS
                    .http_request_duration_seconds
                    .with_label_values(&[&method, &endpoint, &status])
                    .observe(duration);

                METRICS
                    .http_requests_total
                    .with_label_values(&[&method, &endpoint, &status])
                    .inc();
            }

            #[cfg(not(feature = "metrics"))]
            let _ = start; // Suppress unused warning

            result
        })
    }
}

/// Classify request path into endpoint category
///
/// This prevents label cardinality explosion by grouping paths.
#[cfg(feature = "metrics")]
fn classify_endpoint(path: &str) -> String {
    // Note: These prefixes should match the runtime config values
    // For now, use common defaults
    if path.starts_with("/admin") {
        "admin".to_string()
    } else if path.starts_with("/health") {
        "health".to_string()
    } else if path.starts_with("/panel") || path.starts_with("/assets") {
        "frontend".to_string()
    } else {
        // Everything else is a redirect attempt
        "redirect".to_string()
    }
}
