use actix_web::middleware::Next;
use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpResponse,
};
use log::{debug, info};
use std::env;

pub struct AuthMiddleware;

impl AuthMiddleware {
    /// Admin API 身份验证中间件
    pub async fn admin_auth(
        req: ServiceRequest,
        next: Next<BoxBody>,
    ) -> Result<ServiceResponse<BoxBody>, Error> {
        // 在每次请求时重新读取环境变量，而不是在启动时缓存
        let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| "".to_string());

        // 如果 token 为空，认为 Admin API 被禁用
        if admin_token.is_empty() {
            info!("Admin API 访问被拒绝: API 已禁用 (未设置 ADMIN_TOKEN)");
            return Ok(req.into_response(
                HttpResponse::NotFound()
                    .append_header(("Content-Type", "text/html; charset=utf-8"))
                    .append_header(("Connection", "close"))
                    .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                    .body("Not Found"),
            ));
        }

        // 检查 Authorization header
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    if token == admin_token {
                        debug!("Admin API 鉴权成功");
                        return next.call(req).await;
                    }
                }
            }
        }

        info!("Admin API 鉴权失败: token不匹配或缺少Authorization header");
        Ok(req.into_response(
            HttpResponse::Unauthorized()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(serde_json::json!({
                    "code": 401,
                    "data": { "error": "Unauthorized: Invalid or missing token" }
                })),
        ))
    }
}
