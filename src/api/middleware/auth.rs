use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{Method, header::CONTENT_TYPE},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use tracing::{debug, info, trace};

use crate::api::constants;
use crate::api::jwt::get_jwt_service;
use crate::api::services::admin::{ApiResponse, ErrorCode};
use crate::config::{get_runtime_config, keys};
#[cfg(feature = "metrics")]
use crate::metrics_core::MetricsRecorder;

/// 认证方式标记，用于 CSRF 中间件判断是否跳过验证
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuthMethod {
    /// Bearer Token 认证（API 用户，免 CSRF）
    Bearer,
    /// Cookie 认证（Web Panel，需要 CSRF 防护）
    Cookie,
}

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
        let rt = get_runtime_config();
        ready(Ok(AdminAuthMiddleware {
            service: Rc::new(service),
            admin_prefix: rt.get_or(keys::ROUTES_ADMIN_PREFIX, "/admin"),
        }))
    }
}

pub struct AdminAuthMiddleware<S> {
    service: Rc<S>,
    admin_prefix: String,
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
        debug!("Admin token not configured - returning 404");
        req.into_response(
            HttpResponse::NotFound()
                .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                .body("Not Found")
                .map_into_right_body(),
        )
    }

    /// Handle unauthorized requests
    fn handle_unauthorized(req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
        info!("Admin authentication failed - invalid or missing token");
        req.into_response(
            HttpResponse::Unauthorized()
                .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
                .json(ApiResponse::<()> {
                    code: ErrorCode::Unauthorized as i32,
                    message: "Unauthorized: Invalid or missing token".to_string(),
                    data: None,
                })
                .map_into_right_body(),
        )
    }

    /// 从 Authorization header 提取 Bearer token
    fn extract_bearer_token(req: &ServiceRequest) -> Option<String> {
        req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string())
    }

    /// 验证 Bearer token（使用 JWT）
    fn validate_bearer_token(token: &str) -> bool {
        let jwt_service = get_jwt_service();
        match jwt_service.validate_access_token(token) {
            Ok(_claims) => {
                trace!("Bearer token validation successful");
                true
            }
            Err(e) => {
                info!("Bearer token validation failed: {}", e);
                #[cfg(feature = "metrics")]
                crate::metrics::METRICS.inc_auth_failure("bearer");
                false
            }
        }
    }

    /// Validate JWT from Cookie
    fn validate_jwt_cookie(req: &ServiceRequest, cookie_name: &str) -> bool {
        // Try to get the access token from cookie
        let cookie_token = req.cookie(cookie_name).map(|c| c.value().to_string());

        if let Some(token) = cookie_token {
            let jwt_service = get_jwt_service();
            match jwt_service.validate_access_token(&token) {
                Ok(_claims) => {
                    trace!("JWT validation successful");
                    return true;
                }
                Err(e) => {
                    info!("JWT validation failed: {}", e);
                    #[cfg(feature = "metrics")]
                    crate::metrics::METRICS.inc_auth_failure("cookie");
                    return false;
                }
            }
        }

        false
    }

    /// Check if the request path is the login endpoint
    fn is_login_endpoint(req: &ServiceRequest, admin_prefix: &str) -> bool {
        let path = req.path();
        let login_path = format!("{}/v1/auth/login", admin_prefix);
        path == login_path
    }

    /// Check if the request path is the refresh endpoint
    fn is_refresh_endpoint(req: &ServiceRequest, admin_prefix: &str) -> bool {
        let path = req.path();
        let refresh_path = format!("{}/v1/auth/refresh", admin_prefix);
        path == refresh_path
    }

    /// Check if the request path is the logout endpoint
    fn is_logout_endpoint(req: &ServiceRequest, admin_prefix: &str) -> bool {
        let path = req.path();
        let logout_path = format!("{}/v1/auth/logout", admin_prefix);
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
        let admin_prefix = self.admin_prefix.clone();

        Box::pin(async move {
            // 每次请求都读取最新配置
            let rt = get_runtime_config();
            let admin_token = rt.get_or(keys::API_ADMIN_TOKEN, "");

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

            // 1. 先尝试 Bearer Token 认证（API 用户，免 CSRF）
            if let Some(token) = Self::extract_bearer_token(&req)
                && Self::validate_bearer_token(&token)
            {
                trace!("Admin authentication successful via Bearer token");
                // 设置认证方式标记，CSRF 中间件会跳过验证
                req.extensions_mut().insert(AuthMethod::Bearer);
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // 2. 再尝试 Cookie 认证（Web Panel，需要 CSRF 防护）
            if Self::validate_jwt_cookie(&req, constants::ACCESS_COOKIE_NAME) {
                trace!("Admin authentication successful via JWT Cookie");
                // 设置认证方式标记，CSRF 中间件会验证
                req.extensions_mut().insert(AuthMethod::Cookie);
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // 两种认证都失败
            Ok(Self::handle_unauthorized(req))
        })
    }
}
