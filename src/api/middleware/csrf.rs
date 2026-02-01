//! CSRF 防护中间件
//!
//! 双令牌模式：验证 X-CSRF-Token header 与 csrf_token Cookie 是否匹配。
//!
//! 跳过规则：
//! - 安全方法（GET, HEAD, OPTIONS）
//! - Bearer Token 认证（API 用户）
//! - 登录、刷新、登出端点

use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{Method, header::CONTENT_TYPE},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use subtle::ConstantTimeEq;
use tracing::{trace, warn};

use crate::api::constants;
use crate::api::services::admin::{ApiResponse, ErrorCode};
use crate::config::{get_runtime_config, keys};

use super::auth::AuthMethod;

/// CSRF 防护中间件
#[derive(Clone)]
pub struct CsrfGuard;

impl<S, B> Transform<S, ServiceRequest> for CsrfGuard
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = CsrfMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let rt = get_runtime_config();
        let admin_prefix = rt.get_or(keys::ROUTES_ADMIN_PREFIX, "/admin");

        // 预计算认证端点路径（只在初始化时计算一次）
        let auth_endpoints = vec![
            format!("{}/v1/auth/login", admin_prefix),
            format!("{}/v1/auth/refresh", admin_prefix),
            format!("{}/v1/auth/logout", admin_prefix),
        ];

        ready(Ok(CsrfMiddleware {
            service: Rc::new(service),
            auth_endpoints,
        }))
    }
}

pub struct CsrfMiddleware<S> {
    service: Rc<S>,
    auth_endpoints: Vec<String>,
}

impl<S, B> CsrfMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    /// 返回 403 Forbidden 响应
    fn handle_csrf_error(req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
        warn!("CSRF validation failed");
        req.into_response(
            HttpResponse::Forbidden()
                .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
                .json(ApiResponse::<()> {
                    code: ErrorCode::CsrfInvalid as i32,
                    message: "CSRF token missing or invalid".to_string(),
                    data: None,
                })
                .map_into_right_body(),
        )
    }

    /// 检查是否是安全方法（不修改资源）
    fn is_safe_method(method: &Method) -> bool {
        matches!(method, &Method::GET | &Method::HEAD | &Method::OPTIONS)
    }

    /// 检查是否是认证端点（跳过 CSRF）
    /// 使用预计算的认证端点路径，避免每次请求都 format 字符串
    fn is_auth_endpoint(req: &ServiceRequest, auth_endpoints: &[String]) -> bool {
        let path = req.path();
        auth_endpoints.iter().any(|endpoint| endpoint == path)
    }

    /// 常量时间比较两个字符串
    fn constant_time_compare(a: &str, b: &str) -> bool {
        a.as_bytes().ct_eq(b.as_bytes()).into()
    }

    /// 验证 CSRF Token
    fn validate_csrf_token(req: &ServiceRequest) -> bool {
        // 从 Cookie 获取 CSRF token
        let cookie_token = req
            .cookie(constants::CSRF_COOKIE_NAME)
            .map(|c| c.value().to_string());

        // 从 Header 获取 CSRF token
        let header_token = req
            .headers()
            .get("X-CSRF-Token")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        match (cookie_token, header_token) {
            (Some(cookie), Some(header)) => {
                // 常量时间比较，防止时序攻击
                let valid = Self::constant_time_compare(&cookie, &header);
                if !valid {
                    warn!("CSRF token mismatch");
                }
                valid
            }
            (None, _) => {
                warn!("CSRF cookie not found");
                false
            }
            (_, None) => {
                warn!("X-CSRF-Token header not found");
                false
            }
        }
    }
}

impl<S, B> Service<ServiceRequest> for CsrfMiddleware<S>
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
        let auth_endpoints = self.auth_endpoints.clone();

        Box::pin(async move {
            // 1. 跳过安全方法（GET, HEAD, OPTIONS）
            if Self::is_safe_method(req.method()) {
                trace!("CSRF skipped: safe method {}", req.method());
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // 2. 跳过认证端点（login, refresh, logout）
            if Self::is_auth_endpoint(&req, &auth_endpoints) {
                trace!("CSRF skipped: auth endpoint {}", req.path());
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // 3. 跳过 Bearer Token 认证的请求
            // 需要在借用结束前复制出来，避免借用冲突
            let is_bearer_auth = req
                .extensions()
                .get::<AuthMethod>()
                .copied()
                .is_some_and(|m| m == AuthMethod::Bearer);

            if is_bearer_auth {
                trace!("CSRF skipped: Bearer token authentication");
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // 4. Cookie 认证需要验证 CSRF Token
            if !Self::validate_csrf_token(&req) {
                return Ok(Self::handle_csrf_error(req));
            }

            trace!("CSRF validation passed");
            let response = srv.call(req).await?.map_into_left_body();
            Ok(response)
        })
    }
}
