use actix_service::{Service, Transform};
use actix_web::{
    Error, HttpResponse,
    body::EitherBody,
    dev::{ServiceRequest, ServiceResponse},
    http::{Method, header::CONTENT_TYPE},
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;
use subtle::ConstantTimeEq;
use tracing::{info, trace};

use crate::api::constants;
use crate::api::jwt::get_jwt_service;
use crate::api::services::admin::{ApiResponse, ErrorCode};
use crate::config::{get_runtime_config, keys};

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
    type Transform = HealthAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(HealthAuthMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct HealthAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> HealthAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    /// 验证 Authorization header 中的 Bearer token
    fn validate_bearer_token(req: &ServiceRequest, health_token: &str) -> bool {
        if health_token.is_empty() {
            return false;
        }
        if let Some(auth_header) = req.headers().get("Authorization")
            && let Ok(auth_str) = auth_header.to_str()
            && let Some(token) = auth_str.strip_prefix("Bearer ")
            && token.as_bytes().ct_eq(health_token.as_bytes()).into()
        {
            trace!("Health Bearer token validation successful");
            return true;
        }
        false
    }

    /// 验证 JWT Cookie
    fn validate_jwt_cookie(req: &ServiceRequest, cookie_name: &str) -> bool {
        let cookie_token = req.cookie(cookie_name).map(|c| c.value().to_string());
        if let Some(token) = cookie_token {
            let jwt_service = get_jwt_service();
            if jwt_service.validate_access_token(&token).is_ok() {
                trace!("Health JWT validation successful");
                return true;
            }
        }
        false
    }
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
            // 每次请求都读取最新配置
            let rt = get_runtime_config();
            let admin_token = rt.get_or(keys::API_ADMIN_TOKEN, "");
            let health_token = rt.get_or(keys::API_HEALTH_TOKEN, "");

            // 两个 token 都为空才禁用健康接口
            if admin_token.is_empty() && health_token.is_empty() {
                info!("Neither admin_token nor health_token configured - health endpoint disabled");
                return Ok(req.into_response(
                    HttpResponse::NotFound()
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .body("Not Found")
                        .map_into_right_body(),
                ));
            }

            // Handle CORS preflight
            if req.method() == Method::OPTIONS {
                return Ok(req.into_response(
                    HttpResponse::NoContent()
                        .insert_header((CONTENT_TYPE, "text/plain; charset=utf-8"))
                        .finish()
                        .map_into_right_body(),
                ));
            }

            // 认证策略：依次尝试 Bearer token → JWT Cookie
            // 1. 先尝试 Bearer token 认证（给 k8s 等监控工具用）
            //    - 如果 health_token 为空，validate_bearer_token 会返回 false
            //    - 如果 health_token 非空但验证失败，也会返回 false
            if Self::validate_bearer_token(&req, &health_token) {
                trace!("Health authentication successful via Bearer token");
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // 2. 再尝试 JWT Cookie 认证（给前端用户用）
            //    - 使用 admin_token 作为 JWT secret
            if Self::validate_jwt_cookie(&req, constants::ACCESS_COOKIE_NAME) {
                trace!("Health authentication successful via JWT Cookie");
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // 两种认证都失败
            info!("Health authentication failed - invalid or missing token");
            Ok(req.into_response(
                HttpResponse::Unauthorized()
                    .insert_header((CONTENT_TYPE, "application/json; charset=utf-8"))
                    .json(ApiResponse::<()> {
                        code: ErrorCode::Unauthorized as i32,
                        message: "Unauthorized: Invalid or missing token".to_string(),
                        data: None,
                    })
                    .map_into_right_body(),
            ))
        })
    }
}
