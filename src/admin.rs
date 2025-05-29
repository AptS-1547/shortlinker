use actix_web::{HttpResponse, Responder, web, HttpRequest};
use std::env;
use std::sync::{Arc, RwLock};
use log::{debug, info, error};
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::storages::{ShortLink, STORAGE};


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

// 获取 admin 路由前缀
fn get_admin_prefix() -> String {
    env::var("ADMIN_ROUTE_PREFIX").unwrap_or_else(|_| "/admin".to_string())
}

// 鉴权函数
fn check_auth(req: &HttpRequest) -> bool {
    let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| "default_admin_token".to_string());
    
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                return token == admin_token;
            }
        }
    }
    false
}

#[actix_web::route("/admin/link", method = "GET", method = "HEAD")]
async fn get_all_links(req: HttpRequest) -> impl Responder {
    if !check_auth(&req) {
        return auth_error();
    }
    
    let links = STORAGE.load_all().await;
    let serializable_links: HashMap<String, SerializableShortLink> = links.into_iter().map(|(code, link)| {
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
    }).collect();
    HttpResponse::Ok()
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .append_header(("Connection", "close"))
        .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
        .json(serde_json::json!({ "code": 0, "data": Some(serializable_links) }))
}


#[actix_web::route("/admin/link", method = "POST")]
async fn post_link(req: HttpRequest, link: web::Json<PostNewLink>) -> impl Responder {
    if !check_auth(&req) {
        return auth_error();
    }
    
    let new_link = ShortLink {
        code: link.code.clone(),
        target: link.target.clone(),
        created_at: chrono::Utc::now(),
        expires_at: link.expires_at.as_ref().map(|s| chrono::DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&chrono::Utc)),
    };

    match STORAGE.set(new_link.clone()).await {
        Ok(_) => HttpResponse::Created()
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .append_header(("Connection", "close"))
            .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
            .json(ApiResponse {
                code: 0,
                data: PostNewLink {
                    code: new_link.code,
                    target: new_link.target,
                    expires_at: link.expires_at.clone()
                }}),
        Err(e) => HttpResponse::InternalServerError()
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .append_header(("Connection", "close"))
            .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
            .json(ApiResponse {
                code: 1,
                data: serde_json::json!({
                    "error": format!("Error creating link: {}", e),
                }),
            }),
    }
}


#[actix_web::route("/admin/link/{code}", method = "GET", method = "HEAD")]
async fn get_link(req: HttpRequest, code: web::Path<String>) -> impl Responder {
    if !check_auth(&req) {
        return auth_error();
    }
    
    match STORAGE.get(&code).await {
        Some(link) => {
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
                .json(ApiResponse { code: 0, data: serializable_link })
        },
        None => HttpResponse::NotFound()
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .append_header(("Connection", "close"))
            .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
            .json(ApiResponse { code: 1, data: serde_json::json!({ "error": "Link not found" }) }),
    }
}


#[actix_web::route("/admin/link/{code}", method = "DELETE")]
async fn delete_link(req: HttpRequest, code: web::Path<String>) -> impl Responder {
    if !check_auth(&req) {
        return auth_error();
    }
    
    match STORAGE.remove(&code).await {
        Ok(_) => HttpResponse::Ok()
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .append_header(("Connection", "close"))
            .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
            .json(ApiResponse { code: 0, data: serde_json::json!({ "message": "Link deleted successfully" }) }),
        Err(e) => HttpResponse::InternalServerError()
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .append_header(("Connection", "close"))
            .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
            .json(ApiResponse { code: 1, data: serde_json::json!({ "error": format!("Error deleting link: {}", e) }) }),
    }
}

#[actix_web::route("/admin/link/{code}", method = "PUT")]
async fn update_link(req: HttpRequest, code: web::Path<String>, link: web::Json<PostNewLink>) -> impl Responder {
    if !check_auth(&req) {
        return auth_error();
    }
    
    let updated_link = ShortLink {
        code: code.clone(),
        target: link.target.clone(),
        created_at: chrono::Utc::now(),
        expires_at: link.expires_at.as_ref().map(|s| chrono::DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&chrono::Utc)),
    };

    match STORAGE.set(updated_link.clone()).await {
        Ok(_) => HttpResponse::Ok()
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
            }),
        Err(e) => HttpResponse::InternalServerError()
            .append_header(("Content-Type", "application/json; charset=utf-8"))
            .append_header(("Connection", "close"))
            .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
            .json(ApiResponse {
                code: 1,
                data: serde_json::json!({
                    "error": format!("Error updating link: {}", e),
                }),
            }),
    }
}

// 鉴权错误响应
fn auth_error() -> HttpResponse {
    HttpResponse::Unauthorized()
        .append_header(("Content-Type", "application/json; charset=utf-8"))
        .append_header(("Connection", "close"))
        .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
        .json(ApiResponse {
            code: 401,
            data: serde_json::json!({ "error": "Unauthorized: Invalid or missing token" }),
        })
}
