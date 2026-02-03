//! 外部 GeoIP API 实现
//!
//! 使用外部 HTTP API 进行 IP 地理位置查询（如 ip-api.com）
//! 内置 LRU 缓存 + Singleflight 语义，避免重复查询

use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;
use tracing::{trace, warn};

use super::provider::{GeoInfo, GeoIpLookup};

/// GeoIP 缓存 TTL（15 分钟）
const GEOIP_CACHE_TTL_SECS: u64 = 15 * 60;
/// GeoIP 缓存最大容量
const GEOIP_CACHE_MAX_CAPACITY: u64 = 10_000;

/// 外部 API GeoIP Provider
///
/// 内置 Moka 缓存：
/// - LRU 淘汰策略，最大 10000 条
/// - TTL 15 分钟
/// - Singleflight：同一 IP 的并发请求只发一次 HTTP
pub struct ExternalApiProvider {
    client: reqwest::Client,
    api_url_template: String,
    /// IP → GeoInfo 缓存（Option 用于负缓存）
    cache: Cache<String, Option<GeoInfo>>,
}

impl ExternalApiProvider {
    /// 创建外部 API Provider
    ///
    /// `api_url_template` 使用 `{ip}` 作为占位符
    /// 例如: `http://ip-api.com/json/{ip}?fields=countryCode,city`
    pub fn new(api_url_template: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .expect("Failed to create reqwest client");

        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(GEOIP_CACHE_TTL_SECS))
            .max_capacity(GEOIP_CACHE_MAX_CAPACITY)
            .build();

        Self {
            client,
            api_url_template: api_url_template.to_string(),
            cache,
        }
    }

    /// 从外部 API 获取 GeoIP 信息（内部方法）
    async fn fetch_from_api(&self, ip: &str) -> Option<GeoInfo> {
        let url = self.api_url_template.replace("{ip}", ip);

        let resp = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                warn!("GeoIP API request failed for {}: {}", ip, e);
                return None;
            }
        };

        let json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(e) => {
                warn!("GeoIP API response parse failed for {}: {}", ip, e);
                return None;
            }
        };

        // ip-api.com 返回格式: {"countryCode": "CN", "city": "Beijing"}
        // 也支持其他 API 的常见字段名
        let country = json["countryCode"]
            .as_str()
            .or_else(|| json["country_code"].as_str())
            .or_else(|| json["country"].as_str())
            .map(String::from);

        let city = json["city"].as_str().map(String::from);

        trace!(
            "External API lookup for {}: country={:?}, city={:?}",
            ip, country, city
        );

        Some(GeoInfo { country, city })
    }
}

#[async_trait]
impl GeoIpLookup for ExternalApiProvider {
    /// 查询 IP 地理位置（带缓存 + Singleflight）
    ///
    /// - 缓存命中：直接返回
    /// - 缓存未命中：发起 HTTP 请求并缓存结果
    /// - 并发请求同一 IP：只有一个发起请求，其他等待结果
    async fn lookup(&self, ip: &str) -> Option<GeoInfo> {
        let ip_key = ip.to_string();

        // get_with 自带 singleflight 语义：
        // 同一 key 的并发调用只会执行一次闭包，其他等待结果
        // 返回类型是 Option<GeoInfo>（缓存的值类型）
        self.cache
            .get_with(ip_key, async {
                trace!("GeoIP cache miss for {}, fetching from API", ip);
                self.fetch_from_api(ip).await
            })
            .await
    }

    fn name(&self) -> &'static str {
        "ExternalAPI"
    }
}
