// Rust 化完成

use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tracing::debug;

use crate::cache::CacheResult;
use crate::cache::CompositeCacheTrait;
use crate::storages::click::global::get_click_manager;
use crate::storages::{ShortLink, Storage};

static DEFAULT_REDIRECT_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("DEFAULT_URL").unwrap_or_else(|_| "https://esap.cc/repo".to_string())
});

pub struct RedirectService {}

impl RedirectService {
    pub async fn handle_redirect(
        path: web::Path<String>,
        cache: web::Data<Arc<dyn CompositeCacheTrait>>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> impl Responder {
        let captured_path = path.into_inner();

        let response = if captured_path.is_empty() {
            HttpResponse::TemporaryRedirect()
                .insert_header(("Location", DEFAULT_REDIRECT_URL.as_str()))
                .finish()
        } else {
            Self::process_redirect(captured_path, cache, storage).await
        };

        response
    }

    async fn process_redirect(
        capture_path: String,
        cache: web::Data<Arc<dyn CompositeCacheTrait>>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> HttpResponse {
        match cache.get(&capture_path).await {
            CacheResult::Found(link) => {
                Self::update_click(capture_path.clone());
                Self::finish_redirect(link)
            }
            CacheResult::ExistsButNoValue => {
                debug!("L2 cache miss for path: {}", capture_path);
                match storage.get(&capture_path).await {
                    Some(link) => {
                        Self::update_click(capture_path.clone());
                        cache.insert(capture_path.clone(), link.clone()).await;

                        Self::finish_redirect(link)
                    }
                    None => {
                        debug!("Redirect link not found: {}", capture_path);
                        HttpResponse::build(StatusCode::NOT_FOUND)
                            .insert_header(("Content-Type", "text/html; charset=utf-8"))
                            .insert_header(("Cache-Control", "public, max-age=60")) // 缓存404
                            .body("Not Found")
                    }
                }
            }
            CacheResult::NotFound => {
                debug!("Cache not found for path: {}", capture_path);
                HttpResponse::build(StatusCode::NOT_FOUND)
                    .insert_header(("Content-Type", "text/html; charset=utf-8"))
                    .insert_header(("Cache-Control", "public, max-age=60"))
                    .body("Not Found")
            }
        }
    }

    fn update_click(code: String) {
        tokio::spawn(async move {
            match get_click_manager() {
                Some(manager) => {
                    manager.increment(&code);
                }
                None => {
                    debug!("Click manager not initialized, skipping increment for code: {}", code);
                }
            }
        });
    }

    fn finish_redirect(link: ShortLink) -> HttpResponse {
        if let Some(expires_at) = link.expires_at {
            if expires_at < chrono::Utc::now() {
                return HttpResponse::build(StatusCode::NOT_FOUND)
                    .insert_header(("Content-Type", "text/html; charset=utf-8"))
                    .insert_header(("Cache-Control", "public, max-age=60"))
                    .body("Not Found");
            }
        }

        HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
            .insert_header(("Location", link.target))
            .finish()
    }
}
