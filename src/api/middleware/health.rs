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

use crate::api::constants;
use crate::api::jwt::JwtService;
use crate::api::services::admin::{ApiResponse, ErrorCode};
use crate::config::get_config;

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
            && token == health_token
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
            // 每次验证都从最新配置读取 jwt_secret
            let jwt_service = JwtService::from_config();
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
            let config = get_config();
            let admin_token = &config.api.admin_token;
            let health_token = &config.api.health_token;

            // 两个 token 都为空才禁用健康接口
            if admin_token.is_empty() && health_token.is_empty() {
                warn!("Neither admin_token nor health_token configured - health endpoint disabled");
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

            // 先尝试 Bearer token 认证（给 k8s 等监控工具用）
            if Self::validate_bearer_token(&req, health_token) {
                trace!("Health authentication successful via Bearer token");
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // 再尝试 JWT Cookie 认证（给前端用户用）
            if Self::validate_jwt_cookie(&req, constants::ACCESS_COOKIE_NAME) {
                trace!("Health authentication successful via JWT Cookie");
                let response = srv.call(req).await?.map_into_left_body();
                return Ok(response);
            }

            // 两种认证都失败
            warn!("Health authentication failed - invalid or missing token");
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
