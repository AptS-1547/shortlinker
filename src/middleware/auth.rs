use actix_web::middleware::Next;
use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpResponse,
};
use std::env;
use std::sync::OnceLock;
use tracing::{debug, info};

pub struct AuthMiddleware;

static ADMIN_TOKEN: OnceLock<String> = OnceLock::new();

impl AuthMiddleware {
    /// Admin API 身份验证中间件
    pub async fn admin_auth(
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

        // 在第一次调用时从环境变量中获取 ADMIN_TOKEN
        let admin_token = ADMIN_TOKEN.get_or_init(|| env::var("ADMIN_TOKEN").unwrap_or_default());

        // 如果 token 为空，认为 Admin API 被禁用
        if admin_token.is_empty() {
            return Ok(req.into_response(
                HttpResponse::NotFound()
                    .insert_header(("Content-Type", "text/html; charset=utf-8"))
                    .body("Not Found"),
            ));
        }

        // 检查 Authorization header
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Some(auth_bytes) = auth_header.as_bytes().strip_prefix(b"Bearer ") {
                if auth_bytes == admin_token.as_bytes() {
                    debug!("Admin API authentication succeeded");
                    return next.call(req).await;
                }
            }
        }

        info!("Admin API authentication failed: token mismatch or missing Authorization header");
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
