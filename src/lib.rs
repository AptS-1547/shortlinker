//! shortlinker - 短链接服务
//! 
//! 这是一个高性能的短链接服务，支持命令行管理和Web API。

pub mod admin;
pub mod cli;
pub mod reload;
pub mod signal;
pub mod storages;
pub mod utils;

// 重新导出主要的处理函数以供测试使用
use actix_web::{web, HttpResponse, Responder};
use log::{debug, info};
use std::env;

use storages::STORAGE;

/// 短链接重定向处理函数
#[actix_web::route("/{path}*", method = "GET", method = "HEAD")]
pub async fn handle_redirect(path: web::Path<String>) -> impl Responder {
    let captured_path = path.to_string();

    debug!("捕获的路径: {}", captured_path);

    if captured_path.is_empty() {
        let default_url =
            env::var("DEFAULT_URL").unwrap_or_else(|_| "https://esap.cc/repo".to_string());
        info!("重定向到默认主页: {}", default_url);
        HttpResponse::TemporaryRedirect()
            .append_header(("Location", default_url))
            .finish()
    } else {
        // 使用 STORAGE 获取链接
        if let Some(link) = STORAGE.get(&captured_path).await {
            // 检查链接是否过期
            if let Some(expires_at) = link.expires_at {
                if expires_at < chrono::Utc::now() {
                    info!("链接已过期: {}", captured_path);
                    return HttpResponse::NotFound()
                        .append_header(("Content-Type", "text/html; charset=utf-8"))
                        .append_header(("Connection", "close"))
                        .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                        .body("Not Found");
                }
            }

            info!("重定向 {} -> {}", captured_path, link.target);
            HttpResponse::TemporaryRedirect()
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .append_header(("Location", link.target.as_str()))
                .finish()
        } else {
            HttpResponse::NotFound()
                .append_header(("Content-Type", "text/html; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .body("Not Found")
        }
    }
}
