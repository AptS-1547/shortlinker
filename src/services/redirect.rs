// Rust 化完成

use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tracing::debug;
use tracing::instrument;

use crate::cache::Cache;
use crate::cache::CacheResult;
use crate::storages::{ShortLink, Storage};

pub static DEFAULT_REDIRECT_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("DEFAULT_URL").unwrap_or_else(|_| "https://esap.cc/repo".to_string())
});

pub struct RedirectService {}

impl RedirectService {
    #[instrument(skip(cache, storage), fields(path = %path))]
    pub async fn handle_redirect(
        path: web::Path<String>,
        cache: web::Data<Arc<dyn Cache>>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> impl Responder {
        let captured_path = path.into_inner();

        let response = if captured_path.is_empty() {
            HttpResponse::TemporaryRedirect()
                .insert_header(("Location", DEFAULT_REDIRECT_URL.as_str()))
                .insert_header(("Cache-Control", "public, max-age=300"))
                .finish()
        } else {
            Self::process_redirect(captured_path, cache, storage).await
        };

        response
    }

    async fn process_redirect(
        capture_path: String,
        cache: web::Data<Arc<dyn Cache>>,
        storage: web::Data<Arc<dyn Storage>>,
    ) -> HttpResponse {
        match cache.get(&capture_path).await {
            CacheResult::Found(link) => {
                debug!(
                    "L1/L2 Cache hit for path: {} -> {}",
                    capture_path, link.target
                );
                Self::update_click(storage.clone(), capture_path.clone());
                Self::finish_redirect(link)
            }
            CacheResult::ExistsButNoValue => {
                debug!("L2 ache miss for path: {}", capture_path);
                match storage.get(&capture_path).await {
                    Some(link) => {
                        Self::update_click(storage.clone(), capture_path.clone());
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

    fn update_click(storage: web::Data<Arc<dyn Storage>>, code: String) {
        let storage_clone = storage.clone();
        tokio::spawn(async move {
            if let Err(e) = storage_clone.increment_click(&code).await {
                debug!("Failed to increment click for {}: {}", code, e);
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
            .insert_header(("Cache-Control", "public, max-age=60"))
            .finish()
    }
}
