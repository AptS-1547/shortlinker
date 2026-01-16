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

/// 短码最大长度
const MAX_SHORT_CODE_LEN: usize = 128;

/// 验证短码格式：长度 ≤ 128，字符集 [a-zA-Z0-9_.-/]
#[inline]
fn is_valid_short_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= MAX_SHORT_CODE_LEN
        && code.bytes().all(
            |b| matches!(b, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-' | b'.' | b'/'),
        )
}

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
        } else if !is_valid_short_code(&captured_path) {
            // 非法短码，直接 404（不进缓存、不进 DashMap）
            trace!("Invalid short code rejected: {}", &captured_path);
            Self::not_found_response()
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

    /// 更新点击计数（同步无锁操作，无需 spawn）
    #[inline]
    fn update_click(code: &str) {
        if let Some(manager) = get_click_manager() {
            manager.increment(code);
        }
    }

    fn finish_redirect(link: ShortLink) -> HttpResponse {
        HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
            .insert_header(("Location", link.target))
            .finish()
    }
}
