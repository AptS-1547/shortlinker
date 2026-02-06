// Rust 化完成

use std::borrow::Cow;
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
use crate::services::{GeoIpProvider, get_user_agent_store};
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

    /// 从原始数据推导流量来源（用于异步处理）
    #[inline]
    fn derive_source_from_raw(query: &Option<String>, referrer: &Option<String>) -> Option<String> {
        // 1. 检查 utm_source 参数
        if let Some(query) = query
            && let Some(utm_source) = Self::extract_query_param(query, "utm_source")
        {
            return Some(utm_source.into_owned());
        }

        // 2. 有 Referer header → ref:{domain}
        if let Some(referer_url) = referrer
            && let Some(domain) = Self::extract_domain(referer_url)
        {
            return Some(format!("ref:{}", domain));
        }

        // 3. 都没有 → direct
        Some("direct".to_string())
    }

    /// 从 query string 提取指定参数值
    #[inline]
    fn extract_query_param<'a>(query: &'a str, key: &str) -> Option<Cow<'a, str>> {
        for part in query.split('&') {
            if let Some(value) = part.strip_prefix(key).and_then(|s| s.strip_prefix('=')) {
                // urlencoding::decode 返回 Cow，未编码时零分配
                return urlencoding::decode(value).ok();
            }
        }
        None
    }

    /// 从 URL 提取域名
    #[inline]
    fn extract_domain(url: &str) -> Option<&str> {
        // 简单解析：找 :// 后的域名部分
        let without_scheme = url
            .strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url);

        // 取到第一个 / 或 : 或 ? 或 # 为止
        without_scheme
            .split(&['/', ':', '?', '#'][..])
            .next()
            .filter(|s| !s.is_empty())
    }

    /// 更新点击计数（异步处理分析逻辑，不阻塞响应）
    #[inline]
    fn update_click(code: &str, req: &HttpRequest, geoip: Option<web::Data<Arc<GeoIpProvider>>>) {
        let Some(manager) = get_click_manager() else {
            return;
        };

        let rt = get_runtime_config();
        let enable_detailed_logging =
            rt.get_bool_or(keys::ANALYTICS_ENABLE_DETAILED_LOGGING, false);

        if !enable_detailed_logging || !manager.is_detailed_logging_enabled() {
            // 快速路径：只增加 click_count
            manager.increment(code);
            return;
        }

        // 同步阶段：只提取原始字符串
        let code = code.to_string();
        let query = req.uri().query().map(String::from);
        let referrer = req
            .headers()
            .get("referer")
            .and_then(|h| h.to_str().ok())
            .map(String::from);
        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|h| h.to_str().ok())
            .map(String::from);
        let ip = extract_client_ip(req);
        let geoip = geoip.map(|g| Arc::clone(g.get_ref()));
        let enable_ip_logging = rt.get_bool_or(keys::ANALYTICS_ENABLE_IP_LOGGING, true);
        let enable_geo_lookup = rt.get_bool_or(keys::ANALYTICS_ENABLE_GEO_LOOKUP, false);

        // 异步阶段：所有计算都在后台任务执行
        tokio::spawn(async move {
            // derive_source
            let source = Self::derive_source_from_raw(&query, &referrer);

            // UA hash
            let user_agent_hash = user_agent
                .as_ref()
                .and_then(|ua| get_user_agent_store().map(|store| store.get_or_create_hash(ua)));

            let ip_address = if enable_ip_logging { ip.clone() } else { None };

            let mut detail = ClickDetail {
                code,
                timestamp: chrono::Utc::now(),
                referrer,
                user_agent_hash,
                ip_address,
                country: None,
                city: None,
                source,
            };

            // GeoIP 查询（如果启用且有有效 IP）
            if enable_geo_lookup
                && let Some(geoip) = geoip
                && let Some(ref ip_str) = ip
                && let Ok(ip_addr) = ip_str.parse::<IpAddr>()
                && !is_private_or_local(&ip_addr)
                && let Some(geo) = geoip.lookup(ip_str).await
            {
                detail.country = geo.country;
                detail.city = geo.city;
            }

            manager.record_detailed(detail);
        });
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
