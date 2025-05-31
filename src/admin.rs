use actix_web::middleware::Next;
use actix_web::{
    body::BoxBody,
    dev::{ServiceRequest, ServiceResponse},
    web, Error, HttpRequest, HttpResponse, Responder,
};
use log::{debug, error, info};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::storages::{ShortLink, Storage};

// 配置结构体
#[derive(Serialize, Deserialize, Clone, Debug)]
struct SerializableShortLink {
    short_code: String,
    target_url: String,
    created_at: String,
    expires_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ApiResponse<T> {
    code: i32,
    data: T,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PostNewLink {
    code: String,
    target: String,
    expires_at: Option<String>,
}

// 身份验证中间件
pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<BoxBody>,
) -> Result<ServiceResponse<BoxBody>, Error> {
    // 在每次请求时重新读取环境变量，而不是在启动时缓存
    let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| "".to_string());
    debug!(
        "Auth middleware: ADMIN_TOKEN 环境变量结果: {:?}",
        admin_token
    );

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
            .json(ApiResponse {
                code: 401,
                data: serde_json::json!({ "error": "Unauthorized: Invalid or missing token" }),
            }),
    ))
}

#[actix_web::route("/link", method = "GET", method = "HEAD")]
async fn get_all_links(_req: HttpRequest, storage: web::Data<Arc<dyn Storage>>) -> impl Responder {
    info!("Admin API: 获取所有链接请求");

    let links = storage.load_all().await;
    info!("Admin API: 成功获取 {} 个链接", links.len());

    let serializable_links: HashMap<String, SerializableShortLink> = links
        .into_iter()
        .map(|(code, link)| {
            let created_at = link.created_at.to_rfc3339();
            let expires_at = link.expires_at.map(|dt| dt.to_rfc3339());
            (
                code.clone(),
                SerializableShortLink {
                    short_code: code,
                    target_url: link.target,
                    created_at,
                    expires_at,
                },
            )
        })
        .collect();
    HttpResponse::Ok()
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .append_header(("Connection", "close"))
        .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
        .json(serde_json::json!({ "code": 0, "data": Some(serializable_links) }))
}

#[actix_web::route("/link", method = "POST")]
async fn post_link(
    _req: HttpRequest,
    link: web::Json<PostNewLink>,
    storage: web::Data<Arc<dyn Storage>>,
) -> impl Responder {
    info!(
        "Admin API: 创建链接请求 - code: {}, target: {}",
        link.code, link.target
    );

    let new_link = ShortLink {
        code: link.code.clone(),
        target: link.target.clone(),
        created_at: chrono::Utc::now(),
        expires_at: link.expires_at.as_ref().map(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .unwrap()
                .with_timezone(&chrono::Utc)
        }),
    };

    match storage.set(new_link.clone()).await {
        Ok(_) => {
            info!("Admin API: 成功创建链接 - {}", link.code);
            HttpResponse::Created()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(ApiResponse {
                    code: 0,
                    data: PostNewLink {
                        code: new_link.code,
                        target: new_link.target,
                        expires_at: link.expires_at.clone(),
                    },
                })
        }
        Err(e) => {
            error!("Admin API: 创建链接失败 - {}: {}", link.code, e);
            HttpResponse::InternalServerError()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(ApiResponse {
                    code: 1,
                    data: serde_json::json!({
                        "error": format!("Error creating link: {}", e),
                    }),
                })
        }
    }
}

#[actix_web::route("/link/{code}", method = "GET", method = "HEAD")]
async fn get_link(
    _req: HttpRequest,
    code: web::Path<String>,
    storage: web::Data<Arc<dyn Storage>>,
) -> impl Responder {
    info!("Admin API: 获取链接请求 - code: {}", code);

    match storage.get(&code).await {
        Some(link) => {
            info!("Admin API: 成功获取链接 - {}", code);
            let serializable_link = SerializableShortLink {
                short_code: link.code.clone(),
                target_url: link.target,
                created_at: link.created_at.to_rfc3339(),
                expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            };
            HttpResponse::Ok()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(ApiResponse {
                    code: 0,
                    data: serializable_link,
                })
        }
        None => {
            info!("Admin API: 链接不存在 - {}", code);
            HttpResponse::NotFound()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(ApiResponse {
                    code: 1,
                    data: serde_json::json!({ "error": "Link not found" }),
                })
        }
    }
}

#[actix_web::route("/link/{code}", method = "DELETE")]
async fn delete_link(
    _req: HttpRequest,
    code: web::Path<String>,
    storage: web::Data<Arc<dyn Storage>>,
) -> impl Responder {
    info!("Admin API: 删除链接请求 - code: {}", code);

    match storage.remove(&code).await {
        Ok(_) => {
            info!("Admin API: 成功删除链接 - {}", code);
            HttpResponse::Ok()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(ApiResponse {
                    code: 0,
                    data: serde_json::json!({ "message": "Link deleted successfully" }),
                })
        }
        Err(e) => {
            error!("Admin API: 删除链接失败 - {}: {}", code, e);
            HttpResponse::InternalServerError()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(ApiResponse {
                    code: 1,
                    data: serde_json::json!({ "error": format!("Error deleting link: {}", e) }),
                })
        }
    }
}

#[actix_web::route("/link/{code}", method = "PUT")]
async fn update_link(
    _req: HttpRequest,
    code: web::Path<String>,
    link: web::Json<PostNewLink>,
    storage: web::Data<Arc<dyn Storage>>,
) -> impl Responder {
    info!(
        "Admin API: 更新链接请求 - code: {}, target: {}",
        code, link.target
    );

    let updated_link = ShortLink {
        code: code.clone(),
        target: link.target.clone(),
        created_at: chrono::Utc::now(),
        expires_at: link.expires_at.as_ref().map(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .unwrap()
                .with_timezone(&chrono::Utc)
        }),
    };

    match storage.set(updated_link.clone()).await {
        Ok(_) => {
            info!("Admin API: 成功更新链接 - {}", code);
            HttpResponse::Ok()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(ApiResponse {
                    code: 0,
                    data: PostNewLink {
                        code: updated_link.code,
                        target: updated_link.target,
                        expires_at: link.expires_at.clone(),
                    },
                })
        }
        Err(e) => {
            error!("Admin API: 更新链接失败 - {}: {}", code, e);
            HttpResponse::InternalServerError()
                .append_header(("Content-Type", "application/json; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .json(ApiResponse {
                    code: 1,
                    data: serde_json::json!({
                        "error": format!("Error updating link: {}", e),
                    }),
                })
        }
    }
}
