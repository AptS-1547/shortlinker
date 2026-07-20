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
use tracing::{info, trace};

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
        let csrf_names = aster_forge_actix_middleware::csrf::CsrfTokenNames::new(
            constants::CSRF_COOKIE_NAME,
            "X-CSRF-Token",
        )
        .expect("shortlinker CSRF names are static valid header/cookie names");

        ready(Ok(CsrfMiddleware {
            service: Rc::new(service),
            auth_endpoints,
            csrf_names,
        }))
    }
}

pub struct CsrfMiddleware<S> {
    service: Rc<S>,
    auth_endpoints: Vec<String>,
    csrf_names: aster_forge_actix_middleware::csrf::CsrfTokenNames,
}

impl<S, B> CsrfMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    /// 返回 403 Forbidden 响应
    fn handle_csrf_error(req: ServiceRequest) -> ServiceResponse<EitherBody<B>> {
        info!("CSRF validation failed");
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
        let csrf_names = self.csrf_names.clone();

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
            if let Err(error) =
                aster_forge_actix_middleware::csrf::ensure_service_double_submit_token_with_names(
                    &req,
                    &csrf_names,
                )
            {
                info!(kind = ?error.kind(), "CSRF validation failed");
                return Ok(Self::handle_csrf_error(req));
            }

            trace!("CSRF validation passed");
            let response = srv.call(req).await?.map_into_left_body();
            Ok(response)
        })
    }
}
