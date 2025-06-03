use actix_web::{web, HttpResponse, Responder};
use tracing_subscriber::field::debug;
use std::env;
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::debug;

use crate::storages::Storage;

static DEFAULT_URL: OnceLock<String> = OnceLock::new();

pub struct RedirectService;

impl RedirectService {
    pub async fn handle_redirect(
        path: web::Path<String>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> impl Responder {
        let captured_path = path.into_inner();

        debug!("捕获的路径: {}", captured_path);

        if captured_path.is_empty() {
            let default_url = DEFAULT_URL
                .get_or_init(|| {
                    env::var("DEFAULT_URL").unwrap_or_else(|_| "https://esap.cc/repo".to_string())
                })
                .as_str();

            return HttpResponse::TemporaryRedirect()
                .insert_header(("Location", default_url))
                .insert_header(("Cache-Control", "public, max-age=300")) // 5分钟缓存
                .finish();
        }

        match storage.get(&captured_path).await {
            Some(link) => {
                if let Some(expires_at) = link.expires_at {
                    if expires_at < chrono::Utc::now() {
                        return HttpResponse::NotFound()
                            .insert_header(("Content-Type", "text/html; charset=utf-8"))
                            .insert_header(("Cache-Control", "no-cache"))
                            .body("Not Found");
                    }
                }

                let storage_clone = storage.clone();
                let code_clone = captured_path.clone();
                tokio::spawn(async move {
                    let _ = storage_clone.increment_click(&code_clone).await;
                });

                debug!("重定向 {} -> {}", captured_path, link.target);

                HttpResponse::TemporaryRedirect()
                    .insert_header(("Location", link.target))
                    .insert_header(("Cache-Control", "no-cache"))
                    .finish()
            }
            None => {
                debug!("未找到重定向链接: {}", captured_path);
                HttpResponse::NotFound()
                    .insert_header(("Content-Type", "text/html; charset=utf-8"))
                    .insert_header(("Cache-Control", "public, max-age=60")) // 缓存404
                    .body("Not Found")
            }
        }
    }
}
