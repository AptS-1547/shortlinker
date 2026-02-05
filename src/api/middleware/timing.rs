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

/// Drop guard that decrements active connections when dropped.
/// Ensures `dec()` runs even if the future panics.
#[cfg(feature = "metrics")]
struct ActiveConnectionGuard;

#[cfg(feature = "metrics")]
impl Drop for ActiveConnectionGuard {
    fn drop(&mut self) {
        METRICS.http_active_connections.dec();
    }
}

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

        // Extract method and endpoint for labels (avoid String allocation)
        #[cfg(feature = "metrics")]
        let method = method_str(req.method());
        #[cfg(feature = "metrics")]
        let endpoint = classify_endpoint(req.path());

        Box::pin(async move {
            // Guard ensures dec() runs even on panic
            #[cfg(feature = "metrics")]
            METRICS.http_active_connections.inc();
            #[cfg(feature = "metrics")]
            let _guard = ActiveConnectionGuard;

            let result = srv.call(req).await;

            #[cfg(feature = "metrics")]
            {
                let duration = start.elapsed().as_secs_f64();
                let status = match &result {
                    Ok(response) => status_str(response.status()),
                    Err(_) => "500",
                };

                METRICS
                    .http_request_duration_seconds
                    .with_label_values(&[method, endpoint, status])
                    .observe(duration);

                METRICS
                    .http_requests_total
                    .with_label_values(&[method, endpoint, status])
                    .inc();
            }

            #[cfg(not(feature = "metrics"))]
            let _ = start; // Suppress unused warning

            result
        })
    }
}

/// Map HTTP method to a static string (avoids allocation).
#[cfg(feature = "metrics")]
fn method_str(method: &actix_web::http::Method) -> &'static str {
    match method.as_str() {
        "GET" => "GET",
        "POST" => "POST",
        "PUT" => "PUT",
        "DELETE" => "DELETE",
        "HEAD" => "HEAD",
        "OPTIONS" => "OPTIONS",
        "PATCH" => "PATCH",
        _ => "OTHER",
    }
}

/// Map HTTP status code to a static string (avoids allocation for common codes).
#[cfg(feature = "metrics")]
fn status_str(status: actix_web::http::StatusCode) -> &'static str {
    match status.as_u16() {
        200 => "200",
        301 => "301",
        302 => "302",
        304 => "304",
        307 => "307",
        400 => "400",
        401 => "401",
        403 => "403",
        404 => "404",
        405 => "405",
        500 => "500",
        502 => "502",
        503 => "503",
        _ => "other",
    }
}

/// Cached route prefixes for endpoint classification.
/// Initialized once from runtime config (these keys require restart to change).
#[cfg(feature = "metrics")]
struct RoutePrefixes {
    admin: String,
    health: String,
    frontend: String,
}

#[cfg(feature = "metrics")]
static ROUTE_PREFIXES: std::sync::OnceLock<RoutePrefixes> = std::sync::OnceLock::new();

#[cfg(feature = "metrics")]
fn get_route_prefixes() -> &'static RoutePrefixes {
    ROUTE_PREFIXES.get_or_init(|| {
        let rt = crate::config::get_runtime_config();
        RoutePrefixes {
            admin: rt.get_or(crate::config::keys::ROUTES_ADMIN_PREFIX, "/admin"),
            health: rt.get_or(crate::config::keys::ROUTES_HEALTH_PREFIX, "/health"),
            frontend: rt.get_or(crate::config::keys::ROUTES_FRONTEND_PREFIX, "/panel"),
        }
    })
}

/// Classify request path into endpoint category
///
/// This prevents label cardinality explosion by grouping paths.
#[cfg(feature = "metrics")]
fn classify_endpoint(path: &str) -> &'static str {
    let prefixes = get_route_prefixes();
    if path.starts_with(&prefixes.admin) {
        "admin"
    } else if path.starts_with(&prefixes.health) {
        "health"
    } else if path.starts_with(&prefixes.frontend) || path.starts_with("/assets") {
        "frontend"
    } else {
        "redirect"
    }
}
