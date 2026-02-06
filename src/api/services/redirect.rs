// Rust 化完成

use std::borrow::Cow;

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use std::sync::Arc;
use tracing::{debug, error, trace};

use crate::analytics::global::{get_click_manager, is_detailed_logging_stopped};
use crate::cache::CacheResult;
use crate::cache::CompositeCacheTrait;
use crate::config::{get_config, get_runtime_config, keys};
use crate::services::GeoIpProvider;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::utils::ip::extract_client_ip;
use crate::utils::is_valid_short_code;

pub struct RedirectService {}

impl RedirectService {
    pub async fn handle_redirect(
        req: HttpRequest,
        path: web::Path<String>,
        cache: web::Data<Arc<dyn CompositeCacheTrait>>,
        storage: web::Data<Arc<SeaOrmStorage>>,
        geoip: Option<web::Data<Arc<GeoIpProvider>>>,
    ) -> impl Responder {
        let captured_path = path.into_inner();

        if captured_path.is_empty() {
            let rt = get_runtime_config();
            let default_url = rt.get_or(keys::FEATURES_DEFAULT_URL, "https://esap.cc/repo");
            HttpResponse::TemporaryRedirect()
                .insert_header(("Location", default_url))
                .finish()
        } else if !is_valid_short_code(&captured_path) {
            // 非法短码，直接 404（不进缓存、不进 DashMap）
            trace!("Invalid short code rejected: {}", &captured_path);
            Self::not_found_response()
        } else {
            Self::process_redirect(captured_path, req, cache, storage, geoip).await
        }
    }

    async fn process_redirect(
        capture_path: String,
        req: HttpRequest,
        cache: web::Data<Arc<dyn CompositeCacheTrait>>,
        storage: web::Data<Arc<SeaOrmStorage>>,
        geoip: Option<web::Data<Arc<GeoIpProvider>>>,
    ) -> HttpResponse {
        match cache.get(&capture_path).await {
            CacheResult::Found(link) => {
                Self::update_click(&capture_path, &req, geoip);
                Self::finish_redirect(&req, link)
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
                            Self::update_click(&capture_path, &req, geoip);
                            cache.insert(&capture_path, link.clone(), Some(ttl)).await;
                            Self::finish_redirect(&req, link)
                        }
                    },
                    Ok(None) => {
                        debug!("Redirect link not found in database: {}", &capture_path);
                        // Bloom filter false positive: bloom said "maybe exists" but DB says no
                        inc_plain_counter!(
                            crate::metrics::METRICS.bloom_filter_false_positives_total
                        );
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
        inc_counter!(crate::metrics::METRICS.redirects_total, &["404"]);

        HttpResponse::build(StatusCode::NOT_FOUND)
            .insert_header(("Content-Type", "text/html; charset=utf-8"))
            .insert_header(("Cache-Control", "public, max-age=60"))
            .body("Not Found")
    }

    #[inline]
    fn error_response() -> HttpResponse {
        inc_counter!(crate::metrics::METRICS.redirects_total, &["500"]);

        HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .insert_header(("Content-Type", "text/html; charset=utf-8"))
            .body("Internal Server Error")
    }

    /// 更新点击计数（通过 channel 异步处理分析逻辑，不阻塞响应）
    #[inline]
    fn update_click(code: &str, req: &HttpRequest, _geoip: Option<web::Data<Arc<GeoIpProvider>>>) {
        let Some(manager) = get_click_manager() else {
            return;
        };

        let rt = get_runtime_config();
        let enable_detailed_logging =
            rt.get_bool_or(keys::ANALYTICS_ENABLE_DETAILED_LOGGING, false);

        // 检查是否应该停止详细日志（因行数限制）
        if !enable_detailed_logging || !manager.is_detailed_logging_enabled() || is_detailed_logging_stopped() {
            // 快速路径：只增加 click_count
            manager.increment(code);
            return;
        }

        // 采样率检查（在热路径做，避免不必要的字符串 clone）
        let sample_rate = rt.get_f64_or(keys::ANALYTICS_SAMPLE_RATE, 1.0);
        if sample_rate < 1.0 && rand::random::<f64>() >= sample_rate {
            // 不采样，只增加 click_count
            manager.increment(code);
            return;
        }

        // 提取原始数据并发送到 channel（配置读取移到消费者端）
        let event = crate::analytics::RawClickEvent {
            code: code.to_string(),
            query: req.uri().query().map(String::from),
            referrer: req
                .headers()
                .get("referer")
                .and_then(|h| h.to_str().ok())
                .map(String::from),
            user_agent: req
                .headers()
                .get("user-agent")
                .and_then(|h| h.to_str().ok())
                .map(String::from),
            ip: extract_client_ip(req),
        };

        // send_raw_event 内部会调用 increment
        manager.send_raw_event(event);
    }

    fn finish_redirect(req: &HttpRequest, link: ShortLink) -> HttpResponse {
        inc_counter!(crate::metrics::METRICS.redirects_total, &["307"]);

        // 构建目标 URL，可能需要透传 UTM 参数
        let target_url = Self::build_target_url(req, &link.target);

        HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
            .insert_header(("Location", target_url.as_ref()))
            .finish()
    }

    /// 构建目标 URL，根据配置决定是否透传 UTM 参数
    #[inline]
    fn build_target_url<'a>(req: &HttpRequest, target: &'a str) -> Cow<'a, str> {
        let rt = get_runtime_config();
        let enable_passthrough = rt.get_bool_or(keys::UTM_ENABLE_PASSTHROUGH, false);

        if !enable_passthrough {
            return Cow::Borrowed(target);
        }

        // 提取请求中的 UTM 参数
        let Some(query) = req.uri().query() else {
            return Cow::Borrowed(target);
        };

        // 一次遍历提取所有 UTM 参数（返回原始片段）
        let utm_params = Self::extract_utm_params_raw(query);

        if utm_params.is_empty() {
            return Cow::Borrowed(target);
        }

        // 拼接 UTM 参数到目标 URL（直接使用原始片段，零编码开销）
        let separator = if target.contains('?') { "&" } else { "?" };
        let utm_query = utm_params.join("&");

        Cow::Owned(format!("{}{}{}", target, separator, utm_query))
    }

    /// 一次性提取所有 UTM 参数（返回原始片段，零编码开销）
    #[inline]
    fn extract_utm_params_raw(query: &str) -> Vec<&str> {
        const UTM_KEYS: [&str; 5] = [
            "utm_source",
            "utm_medium",
            "utm_campaign",
            "utm_term",
            "utm_content",
        ];

        query
            .split('&')
            .filter(|part| {
                part.find('=')
                    .map(|pos| UTM_KEYS.contains(&&part[..pos]))
                    .unwrap_or(false)
            })
            .collect()
    }
}

/// Redirect 路由配置
pub fn redirect_routes() -> actix_web::Scope {
    use actix_web::web;

    web::scope("")
        .route("/{path}*", web::get().to(RedirectService::handle_redirect))
        .route("/{path}*", web::head().to(RedirectService::handle_redirect))
}
