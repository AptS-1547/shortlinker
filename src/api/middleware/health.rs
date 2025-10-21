use actix_service::{Service, Transform};
use actix_web::{Error, body::EitherBody, dev::ServiceRequest, dev::ServiceResponse};
use futures_util::future::{Ready, ready};
use std::rc::Rc;
use tracing::trace;

use super::common::{AuthStrategy, GenericAuthMiddleware, extract_token_from_service_request};

/// Health check authentication strategy
#[derive(Clone)]
pub struct HealthAuthStrategy {
    health_token: String,
    admin_token: String,
}

impl HealthAuthStrategy {
    pub fn new(health_token: String, admin_token: String) -> Self {
        Self {
            health_token,
            admin_token,
        }
    }
}

impl AuthStrategy for HealthAuthStrategy {
    fn requires_auth(&self, _req: &ServiceRequest) -> bool {
        // If both tokens are empty, health endpoints are disabled
        !self.health_token.is_empty() || !self.admin_token.is_empty()
    }

    fn validate_request(&self, req: &ServiceRequest) -> bool {
        // Try to extract token from cookie first (sl_admin), then fallback to bearer header
        let token = extract_token_from_service_request(req, "sl_admin").or_else(|| {
            // Fallback to Bearer token if cookie not found
            req.headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|val| val.to_string())
        });

        match token {
            Some(t) if t == self.health_token || t == self.admin_token => {
                trace!("Health token validation successful");
                true
            }
            _ => {
                trace!("Health token validation failed");
                false
            }
        }
    }

    fn name(&self) -> &'static str {
        "HealthAuth"
    }
}

/// Health check authentication middleware
#[derive(Clone)]
pub struct HealthAuth;

impl<S, B> Transform<S, ServiceRequest> for HealthAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = GenericAuthMiddleware<S, HealthAuthStrategy>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let config = crate::system::app_config::get_config();
        let strategy = HealthAuthStrategy::new(
            config.api.health_token.clone(),
            config.api.admin_token.clone(),
        );

        ready(Ok(GenericAuthMiddleware {
            service: Rc::new(service),
            strategy,
        }))
    }
}

pub struct HealthAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for HealthAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
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
        Box::pin(async move {
            let config = crate::config::get_config();
            let token = HEALTH_TOKEN.get_or_init(|| config.api.health_token.clone());
            let admin_token = ADMIN_TOKEN.get_or_init(|| config.api.admin_token.clone());

            if token.is_empty() && admin_token.is_empty() {
                return Ok(req.into_response(
                    HttpResponse::build(StatusCode::NOT_FOUND)
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .body("Not Found")
                        .map_into_right_body(),
                ));
            }

            if req.method().clone() == actix_web::http::Method::OPTIONS {
                return Ok(req.into_response(
                    HttpResponse::build(StatusCode::NO_CONTENT)
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .finish()
                        .map_into_right_body(),
                ));
            }

            let auth_ok = req
                .headers()
                .get("Authorization")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|val| val == token || val == admin_token)
                .unwrap_or(false);

            if !auth_ok {
                info!("Health auth failed");
                return Ok(req.into_response(
                    HttpResponse::build(StatusCode::UNAUTHORIZED)
                        .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
                        .json(serde_json::json!({
                            "code": 401,
                            "data": { "error": "Unauthorized: Invalid or missing token" }
                        }))
                        .map_into_right_body(),
                ));
            }

            trace!("Health auth passed");
            let res = srv.call(req).await?.map_into_left_body();
            Ok(res)
        })
    }
}
