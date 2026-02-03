// Rust 化完成

use std::net::IpAddr;

use actix_web::http::StatusCode;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use std::sync::Arc;
use tracing::{debug, error, trace};

use crate::analytics::ClickDetail;
use crate::analytics::global::get_click_manager;
use crate::cache::CacheResult;
use crate::cache::CompositeCacheTrait;
use crate::config::{get_config, get_runtime_config, keys};
use crate::services::GeoIpProvider;
use crate::storage::{SeaOrmStorage, ShortLink};
use crate::utils::ip::{extract_client_ip, is_private_or_local};
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
                            Self::update_click(&capture_path, &req, geoip);
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

    /// 提取点击详细信息
    fn extract_click_detail(code: &str, req: &HttpRequest) -> ClickDetail {
        let rt = get_runtime_config();
        let enable_ip_logging = rt.get_bool_or(keys::ANALYTICS_ENABLE_IP_LOGGING, true);

        ClickDetail {
            code: code.to_string(),
            timestamp: chrono::Utc::now(),
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
            ip_address: if enable_ip_logging {
                extract_client_ip(req)
            } else {
                None
            },
            country: None,
            city: None,
        }
    }

    /// 更新点击计数（同步无锁操作，无需 spawn）
    #[inline]
    fn update_click(code: &str, req: &HttpRequest, geoip: Option<web::Data<Arc<GeoIpProvider>>>) {
        let Some(manager) = get_click_manager() else {
            return;
        };

        let rt = get_runtime_config();
        let enable_detailed_logging =
            rt.get_bool_or(keys::ANALYTICS_ENABLE_DETAILED_LOGGING, false);

        if enable_detailed_logging && manager.is_detailed_logging_enabled() {
            let detail = Self::extract_click_detail(code, req);

            // 异步 GeoIP 查询（不阻塞响应）
            let enable_geo_lookup = rt.get_bool_or(keys::ANALYTICS_ENABLE_GEO_LOOKUP, false);
            if enable_geo_lookup
                && let Some(ref geoip_provider) = geoip
                && let Some(ref ip) = detail.ip_address
            {
                // 私有/本地 IP 不查 GeoIP（查了也没意义）
                if let Ok(ip_addr) = ip.parse::<IpAddr>()
                    && is_private_or_local(&ip_addr)
                {
                    trace!("Skipping GeoIP lookup for private/local IP: {}", ip);
                    manager.record_detailed(detail);
                    return;
                }

                let ip = ip.clone();
                let geoip = Arc::clone(geoip_provider.get_ref());
                let manager = Arc::clone(manager);

                // 异步处理 GeoIP 查询和记录
                tokio::spawn(async move {
                    let geo = geoip.lookup(&ip).await;
                    let detail = detail.with_geo(
                        geo.as_ref().and_then(|g| g.country.clone()),
                        geo.as_ref().and_then(|g| g.city.clone()),
                    );
                    manager.record_detailed(detail);
                });
                return;
            }

            // 无需 GeoIP 查询，直接记录
            manager.record_detailed(detail);
        } else {
            // 只增加 click_count（现有逻辑）
            manager.increment(code);
        }
    }

    fn finish_redirect(link: ShortLink) -> HttpResponse {
        HttpResponse::build(StatusCode::TEMPORARY_REDIRECT)
            .insert_header(("Location", link.target))
            .finish()
    }
}

/// Redirect 路由配置
pub fn redirect_routes() -> actix_web::Scope {
    use actix_web::web;

    web::scope("")
        .route("/{path}*", web::get().to(RedirectService::handle_redirect))
        .route("/{path}*", web::head().to(RedirectService::handle_redirect))
}
