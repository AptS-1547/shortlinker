// Rust 化完成

use actix_web::http::StatusCode;
use actix_web::{HttpResponse, Responder, web};
use std::sync::Arc;
use tracing::{debug, error, trace};

use crate::analytics::global::get_click_manager;
use crate::cache::CacheResult;
use crate::cache::CompositeCacheTrait;
use crate::config::get_config;
use crate::storage::{SeaOrmStorage, ShortLink};

pub struct RedirectService {}

impl RedirectService {
    pub async fn handle_redirect(
        path: web::Path<String>,
        cache: web::Data<Arc<dyn CompositeCacheTrait>>,
        storage: web::Data<Arc<SeaOrmStorage>>,
    ) -> impl Responder {
        let captured_path = path.into_inner();

        if captured_path.is_empty() {
            let default_url = get_config().features.default_url.clone();
            HttpResponse::TemporaryRedirect()
                .insert_header(("Location", default_url))
                .finish()
        } else {
            Self::process_redirect(captured_path, cache, storage).await
        }
    }

    async fn process_redirect(
        capture_path: String,
        cache: web::Data<Arc<dyn CompositeCacheTrait>>,
        storage: web::Data<Arc<SeaOrmStorage>>,
    ) -> HttpResponse {
        match cache.get(&capture_path).await {
            CacheResult::Found(link) => {
                Self::update_click(&capture_path);
                Self::finish_redirect(link)
            }
            CacheResult::Miss => {
                trace!("Cache miss for path: {}", &capture_path);
                match storage.get(&capture_path).await {
                    Ok(Some(link)) => match link.cache_ttl(get_config().cache.default_ttl) {
                        None => {
                            debug!("Expired link from storage: {}", &capture_path);
                            cache.mark_not_found(&capture_path).await;
                            Self::not_found_response()
                        }
                        Some(ttl) => {
                            Self::update_click(&capture_path);
                            cache.insert(&capture_path, link.clone(), Some(ttl)).await;
                            Self::finish_redirect(link)
                        }
                    },
                    Ok(None) => {
                        debug!("Redirect link not found in database: {}", &capture_path);
                        cache.mark_not_found(&capture_path).await;
                        Self::not_found_response()
                    }
                    Err(e) => {
                        error!("Database error during redirect lookup: {}", e);
                        Self::error_response()
                    }
                }
            }
            CacheResult::NotFound => {
                debug!("Cache not found for path: {}", &capture_path);
                Self::not_found_response()
            }
        }
    }

    #[inline]
    fn not_found_response() -> HttpResponse {
        HttpResponse::build(StatusCode::NOT_FOUND)
            .insert_header(("Content-Type", "text/html; charset=utf-8"))
            .insert_header(("Cache-Control", "public, max-age=60"))
            .body("Not Found")
    }

    #[inline]
    fn error_response() -> HttpResponse {
        HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .insert_header(("Content-Type", "text/html; charset=utf-8"))
            .body("Internal Server Error")
    }

    fn update_click(code: &str) {
        let code: Arc<str> = Arc::from(code);
        tokio::spawn(async move {
            match get_click_manager() {
                Some(manager) => {
                    manager.increment(&code);
                }
                None => {
                    trace!(
                        "Click manager not initialized, skipping increment for code: {}",
                        code
                    );
                }
            }
        });
    }

    fn finish_redirect(link: ShortLink) -> HttpResponse {
        HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
            .insert_header(("Location", link.target))
            .finish()
    }
}
