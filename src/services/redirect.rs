use actix_web::{web, HttpResponse, Responder};
use std::env;
use std::sync::Arc;
use tracing::{debug, info};

use crate::storages::Storage;

pub struct RedirectService;

impl RedirectService {
    pub async fn handle_redirect(
        path: web::Path<String>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> impl Responder {
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
            // 使用注入的 storage 获取链接
            if let Some(link) = storage.get(&captured_path).await {
                // 检查链接是否过期
                if let Some(expires_at) = link.expires_at {
                    if expires_at < chrono::Utc::now() {
                        info!("链接已过期: {}", captured_path);
                        return HttpResponse::NotFound()
                            .append_header(("Content-Type", "text/html; charset=utf-8"))
                            .append_header(("Connection", "keep-alive"))
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
                    .append_header(("Connection", "keep-alive"))
                    .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                    .body("Not Found")
            }
        }
    }
}
