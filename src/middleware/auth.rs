use actix_service::{Service, Transform};
use actix_web::{
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{header::CONTENT_TYPE, Method},
    Error, HttpResponse,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::{env, rc::Rc};
use tracing::{debug, warn};

/// Admin authentication middleware
#[derive(Clone)]
pub struct AdminAuth;

impl<S, B> Transform<S, ServiceRequest> for AdminAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AdminAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AdminAuthMiddleware {
            service: Rc::new(service),
            admin_prefix: env::var("ADMIN_PREFIX").unwrap_or_else(|_| "admin".to_string()),
            admin_token: env::var("ADMIN_TOKEN").unwrap_or_else(|_| "".to_string()),
        }))
    }
}

pub struct AdminAuthMiddleware<S> {
    service: Rc<S>,
    admin_prefix: String,
    admin_token: String,
}

impl<S, B> AdminAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    /// Handle OPTIONS requests for CORS preflight
    fn handle_options_request(req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
        req.into_response(
            HttpResponse::NoContent()
                .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                .finish()
                .map_into_right_body(),
        )
    }

    /// Handle requests when admin token is not configured
    fn handle_missing_token(req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
        warn!("Admin token not configured - returning 404");
        req.into_response(
            HttpResponse::NotFound()
                .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                .body("Not Found")
                .map_into_right_body(),
        )
    }

    /// Handle unauthorized requests
    fn handle_unauthorized(req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
        warn!("Admin authentication failed - invalid or missing token");
        req.into_response(
            HttpResponse::Unauthorized()
                .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
                .json(serde_json::json!({
                    "code": 401,
                    "data": { "error": "Unauthorized: Invalid or missing token" }
                }))
                .map_into_right_body(),
        )
    }

    /// Extract and validate the bearer token
    fn validate_bearer_token(req: &ServiceRequest, expected_token: &str) -> bool {
        req.headers()
            .get("Authorization")
            .and_then(|header_value| header_value.to_str().ok())
            .and_then(|auth_str| auth_str.strip_prefix("Bearer "))
            .map(|token| token == expected_token)
            .unwrap_or(false)
    }

    /// Check if the request path is the login endpoint
    fn is_login_endpoint(req: &ServiceRequest, admin_prefix: &str) -> bool {
        let path = req.path();
        let login_path = format!("/{}/auth/login", admin_prefix);
        path == login_path
    }
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddleware<S>
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
        let admin_token = self.admin_token.clone();
        let admin_prefix = self.admin_prefix.clone();
        
        Box::pin(async move {
            // Handle CORS preflight requests
            if req.method() == Method::OPTIONS {
                return Ok(Self::handle_options_request(req));
            }

            // Check if admin token is configured
            if admin_token.is_empty() {
                return Ok(Self::handle_missing_token(req));
            }

            // Allow login endpoint to pass through without authentication
            if Self::is_login_endpoint(&req, &admin_prefix) {
                debug!("Login endpoint accessed - bypassing authentication");
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // Validate the bearer token for all other endpoints
            if !Self::validate_bearer_token(&req, &admin_token) {
                return Ok(Self::handle_unauthorized(req));
            }

            debug!("Admin authentication successful");
            
            // Process the request with the next service
            let response = srv.call(req).await?.map_into_left_body();
            Ok(response)
        })
    }
}
