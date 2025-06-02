use actix_web::middleware::Next;
use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpResponse,
};
use std::env;
use tracing::{debug, info};

pub struct HealthMiddleware;

impl HealthMiddleware {
    /// 严格的健康检查中间件
    /// 当设置了特定的健康检查 token 时进行验证
    pub async fn health_auth(
        req: ServiceRequest,
        next: Next<BoxBody>,
    ) -> Result<ServiceResponse<BoxBody>, Error> {
        // 检查是否设置了健康检查 token
        let health_token = env::var("HEALTH_TOKEN").unwrap_or_default();

        // 如果 token 为空，认为 Health API 被禁用
        if health_token.is_empty() {
            info!("Health API 访问被拒绝: API 已禁用 (未设置 HEALTH_TOKEN)");
            return Ok(req.into_response(
                HttpResponse::NotFound()
                    .append_header(("Content-Type", "text/html; charset=utf-8"))
                    .append_header(("Connection", "keep-alive"))
                    .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                    .body("Not Found"),
            ));
        }

        debug!("Health auth: token validation required");

        // 检查 Authorization header
        if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    if token == health_token {
                        debug!("Health API 鉴权成功");
                        return next.call(req).await;
                    }
                }
            }
        }

        info!("Health API 鉴权失败: token不匹配或缺少Authorization header");
        Ok(req.into_response(
            HttpResponse::Unauthorized()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "keep-alive"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(serde_json::json!({
                    "code": 401,
                    "data": { "error": "Unauthorized: Invalid or missing token" }
                })),
        ))
    }
}
