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
        // 在每次请求时重新读取环境变量，而不是在启动时缓存
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
                    debug!("Admin API 鉴权成功");
                    return next.call(req).await;
                }
            }
        }

        info!("Admin API 鉴权失败: token不匹配或缺少Authorization header");
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
