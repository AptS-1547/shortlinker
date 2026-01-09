use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{Method, header::CONTENT_TYPE},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use tracing::{trace, warn};

use crate::api::jwt::JwtService;

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
        let config = crate::config::get_config();
        ready(Ok(AdminAuthMiddleware {
            service: Rc::new(service),
            admin_prefix: config.routes.admin_prefix.clone(),
            admin_token: config.api.admin_token.clone(),
            access_cookie_name: config.api.access_cookie_name.clone(),
            jwt_secret: config.api.jwt_secret.clone(),
            access_token_minutes: config.api.access_token_minutes,
            refresh_token_days: config.api.refresh_token_days,
        }))
    }
}

pub struct AdminAuthMiddleware<S> {
    service: Rc<S>,
    admin_prefix: String,
    admin_token: String,
    access_cookie_name: String,
    jwt_secret: String,
    access_token_minutes: u64,
    refresh_token_days: u64,
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

    /// Validate JWT from Cookie
    fn validate_jwt_cookie(
        req: &ServiceRequest,
        cookie_name: &str,
        jwt_secret: &str,
        access_token_minutes: u64,
        refresh_token_days: u64,
    ) -> bool {
        // Try to get the access token from cookie
        let cookie_token = req.cookie(cookie_name).map(|c| c.value().to_string());

        if let Some(token) = cookie_token {
            let jwt_service = JwtService::new(jwt_secret, access_token_minutes, refresh_token_days);
            match jwt_service.validate_access_token(&token) {
                Ok(_claims) => {
                    trace!("JWT validation successful");
                    return true;
                }
                Err(e) => {
                    warn!("JWT validation failed: {}", e);
                    return false;
                }
            }
        }

        false
    }

    /// Check if the request path is the login endpoint
    fn is_login_endpoint(req: &ServiceRequest, admin_prefix: &str) -> bool {
        let path = req.path();
        let login_path = format!("{}/auth/login", admin_prefix);
        path == login_path
    }

    /// Check if the request path is the refresh endpoint
    fn is_refresh_endpoint(req: &ServiceRequest, admin_prefix: &str) -> bool {
        let path = req.path();
        let refresh_path = format!("{}/auth/refresh", admin_prefix);
        path == refresh_path
    }

    /// Check if the request path is the logout endpoint
    fn is_logout_endpoint(req: &ServiceRequest, admin_prefix: &str) -> bool {
        let path = req.path();
        let logout_path = format!("{}/auth/logout", admin_prefix);
        path == logout_path
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
        let access_cookie_name = self.access_cookie_name.clone();
        let jwt_secret = self.jwt_secret.clone();
        let access_token_minutes = self.access_token_minutes;
        let refresh_token_days = self.refresh_token_days;

        Box::pin(async move {
            // Check if admin token is configured
            if admin_token.is_empty() {
                return Ok(Self::handle_missing_token(req));
            }

            // Handle CORS preflight requests
            if req.method() == Method::OPTIONS {
                return Ok(Self::handle_options_request(req));
            }

            // Allow login endpoint to pass through without authentication
            if Self::is_login_endpoint(&req, &admin_prefix) {
                trace!("Login endpoint accessed - bypassing authentication");
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // Allow refresh endpoint to pass through (it validates refresh token internally)
            if Self::is_refresh_endpoint(&req, &admin_prefix) {
                trace!("Refresh endpoint accessed - bypassing access token check");
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // Allow logout endpoint to pass through
            if Self::is_logout_endpoint(&req, &admin_prefix) {
                trace!("Logout endpoint accessed - bypassing authentication");
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // Validate JWT from cookie
            if !Self::validate_jwt_cookie(
                &req,
                &access_cookie_name,
                &jwt_secret,
                access_token_minutes,
                refresh_token_days,
            ) {
                return Ok(Self::handle_unauthorized(req));
            }

            trace!("Admin authentication successful via JWT Cookie");

            // Process the request with the next service
            let response = srv.call(req).await?.map_into_left_body();
            Ok(response)
        })
    }
}
