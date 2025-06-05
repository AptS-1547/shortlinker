use actix_web::middleware::Next;
use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpResponse,
};
use std::env;
use std::sync::OnceLock;
use tracing::{debug, info};

static HEALTH_TOKEN: OnceLock<String> = OnceLock::new();

pub struct HealthMiddleware;

impl HealthMiddleware {
    /// 严格的健康检查中间件
    /// 当设置了特定的健康检查 token 时进行验证
    pub async fn health_auth(
        req: ServiceRequest,
        next: Next<BoxBody>,
    ) -> Result<ServiceResponse<BoxBody>, Error> {
        if req.method() == actix_web::http::Method::OPTIONS {
            // 对于 OPTIONS 请求，直接返回 204 No Content
            return Ok(req.into_response(
                HttpResponse::NoContent()
                    .insert_header(("Content-Type", "text/html; charset=utf-8"))
                    .finish(),
            ));
        }

        // 检查是否设置了健康检查 token
        let health_token =
            HEALTH_TOKEN.get_or_init(|| env::var("HEALTH_TOKEN").unwrap_or_default());

        // 如果 token 为空，认为 Health API 被禁用
        if health_token.is_empty() {
            return Ok(req.into_response(
                HttpResponse::NotFound()
                    .insert_header(("Content-Type", "text/html; charset=utf-8"))
                    .body("Not Found"),
            ));
        }

        // 检查 Authorization header
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Some(auth_bytes) = auth_header.as_bytes().strip_prefix(b"Bearer ") {
                if auth_bytes == health_token.as_bytes() {
                    debug!("Health API authentication succeeded");
                    return next.call(req).await;
                }
            }
        }

        info!("Health API authentication failed: token mismatch or missing Authorization header");
        Ok(req.into_response(
            HttpResponse::Unauthorized()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .json(serde_json::json!({
                    "code": 401,
                    "data": { "error": "Unauthorized: Invalid or missing token" }
                })),
        ))
    }
}
