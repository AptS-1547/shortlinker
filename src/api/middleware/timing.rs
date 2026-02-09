//! HTTP timing middleware
//!
//! Records HTTP request duration, request count, and active connections
//! via the `MetricsRecorder` trait (noop when metrics feature is disabled).

use actix_service::{Service, Transform};
use actix_web::{
    Error,
    dev::{ServiceRequest, ServiceResponse},
    web,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use crate::metrics_core::MetricsRecorder;

/// Drop guard that decrements active connections when dropped.
/// Ensures `dec()` runs even if the future panics.
struct ActiveConnectionGuard {
    metrics: Arc<dyn MetricsRecorder>,
}

impl Drop for ActiveConnectionGuard {
    fn drop(&mut self) {
        self.metrics.dec_active_connections();
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
        let method = method_str(req.method());
        let endpoint = classify_endpoint(req.path());

        // Extract metrics from app data
        let metrics: Arc<dyn MetricsRecorder> = req
            .app_data::<web::Data<Arc<dyn MetricsRecorder>>>()
            .map(|d| d.get_ref().clone())
            .unwrap_or_else(|| crate::metrics_core::NoopMetrics::arc());

        Box::pin(async move {
            // Guard ensures dec() runs even on panic
            metrics.inc_active_connections();
            let _guard = ActiveConnectionGuard {
                metrics: metrics.clone(),
            };

            let result = srv.call(req).await;

            let duration = start.elapsed().as_secs_f64();
            let status = match &result {
                Ok(response) => status_str(response.status()),
                Err(_) => "500",
            };

            metrics.observe_http_request(method, endpoint, status, duration);
            metrics.inc_http_request(method, endpoint, status);

            result
        })
    }
}

/// Map HTTP method to a static string (avoids allocation).
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
struct RoutePrefixes {
    admin: String,
    health: String,
    frontend: String,
}

static ROUTE_PREFIXES: std::sync::OnceLock<RoutePrefixes> = std::sync::OnceLock::new();

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
