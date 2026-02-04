//! 外部 GeoIP API 实现
//!
//! 使用外部 HTTP API 进行 IP 地理位置查询（如 ip-api.com）
//! 内置 LRU 缓存 + Singleflight 语义，避免重复查询

use std::sync::OnceLock;
use std::time::Duration;

use async_trait::async_trait;
use moka::future::Cache;
use tracing::{trace, warn};
use ureq::Agent;

use super::provider::{GeoInfo, GeoIpLookup};

/// GeoIP 缓存 TTL（15 分钟）
const GEOIP_CACHE_TTL_SECS: u64 = 15 * 60;
/// GeoIP 缓存最大容量
const GEOIP_CACHE_MAX_CAPACITY: u64 = 10_000;
/// HTTP 请求超时时间
const HTTP_TIMEOUT_SECS: u64 = 2;

/// 全局 HTTP Agent（ureq 的 Agent 是 Send + Sync）
static HTTP_AGENT: OnceLock<Agent> = OnceLock::new();

fn get_agent() -> &'static Agent {
    HTTP_AGENT.get_or_init(|| {
        Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(HTTP_TIMEOUT_SECS)))
            .build()
            .into()
    })
}

/// 外部 API GeoIP Provider
///
/// 内置 Moka 缓存：
/// - LRU 淘汰策略，最大 10000 条
/// - TTL 15 分钟
/// - Singleflight：同一 IP 的并发请求只发一次 HTTP
pub struct ExternalApiProvider {
    api_url_template: String,
    /// IP → GeoInfo 缓存（Option 用于负缓存）
    cache: Cache<String, Option<GeoInfo>>,
}

impl ExternalApiProvider {
    /// 创建外部 API Provider
    ///
    /// `api_url_template` 使用 `{ip}` 作为占位符
    /// 例如: `http://ip-api.com/json/{ip}?fields=status,countryCode,city`
    pub fn new(api_url_template: &str) -> Self {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(GEOIP_CACHE_TTL_SECS))
            .max_capacity(GEOIP_CACHE_MAX_CAPACITY)
            .build();

        Self {
            api_url_template: api_url_template.to_string(),
            cache,
        }
    }

    /// 从外部 API 获取 GeoIP 信息（同步，在 spawn_blocking 中调用）
    fn fetch_from_api_sync(url: String) -> Option<GeoInfo> {
        let agent = get_agent();

        let resp = match agent.get(&url).call() {
            Ok(r) => r,
            Err(e) => {
                warn!("GeoIP API request to \"{}\" failed: {}", url, e);
                return None;
            }
        };

        let json: serde_json::Value = match resp.into_body().read_json() {
            Ok(j) => j,
            Err(e) => {
                warn!("GeoIP API response from \"{}\" parse failed: {}", url, e);
                return None;
            }
        };

        // ip-api.com 返回格式: {"countryCode": "CN", "city": "Beijing"}
        // 失败时返回: {"status": "fail", ...}
        // 也支持其他 API 的常见字段名
        if json["status"].as_str() == Some("fail") {
            trace!("External API returned fail status");
            return None;
        }

        let country = json["countryCode"]
            .as_str()
            .or_else(|| json["country_code"].as_str())
            .or_else(|| json["country"].as_str())
            .map(String::from);

        let city = json["city"].as_str().map(String::from);

        trace!(
            "External API lookup: country={:?}, city={:?}",
            country, city
        );

        Some(GeoInfo { country, city })
    }

    /// 从外部 API 获取 GeoIP 信息（异步包装）
    async fn fetch_from_api(&self, ip: &str) -> Option<GeoInfo> {
        let url = self.api_url_template.replace("{ip}", ip);

        // 使用 spawn_blocking 在线程池中执行同步 HTTP 请求
        tokio::task::spawn_blocking(move || Self::fetch_from_api_sync(url))
            .await
            .unwrap_or_else(|e| {
                warn!("GeoIP spawn_blocking failed: {}", e);
                None
            })
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

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试 ureq 基本 HTTP 请求
    /// 依赖外部网络服务，CI 环境可能失败
    #[test]
    #[ignore]
    fn test_ureq_basic_request() {
        let agent = get_agent();

        // 用 httpbin 测试基本连通性
        let resp = agent.get("https://httpbin.org/json").call();

        assert!(resp.is_ok(), "HTTP request should succeed");

        let resp = resp.unwrap();
        assert_eq!(resp.status(), 200);

        let json: serde_json::Value = resp.into_body().read_json().unwrap();
        assert!(json.is_object(), "Response should be JSON object");
    }

    /// 测试 fetch_from_api_sync 解析 ip-api.com 格式
    /// 依赖外部网络服务，CI 环境可能失败
    #[test]
    #[ignore]
    fn test_fetch_from_api_sync_real() {
        // 用 Google DNS 的 IP 测试（稳定、公开）
        let url = "http://ip-api.com/json/8.8.8.8?fields=status,countryCode,city".to_string();

        let result = ExternalApiProvider::fetch_from_api_sync(url);

        assert!(result.is_some(), "Should get GeoIP result for 8.8.8.8");

        let geo = result.unwrap();
        assert_eq!(
            geo.country,
            Some("US".to_string()),
            "Google DNS should be in US"
        );
        // city 可能是空的，不强制断言
    }

    /// 测试 ExternalApiProvider 完整流程（含缓存）
    /// 依赖外部网络服务，CI 环境可能失败
    #[tokio::test]
    #[ignore]
    async fn test_external_api_provider_lookup() {
        let provider =
            ExternalApiProvider::new("http://ip-api.com/json/{ip}?fields=status,countryCode,city");

        // 第一次查询（缓存未命中，发起 HTTP 请求）
        let result1 = provider.lookup("8.8.8.8").await;
        assert!(result1.is_some(), "First lookup should succeed");
        assert_eq!(result1.as_ref().unwrap().country, Some("US".to_string()));

        // 第二次查询（缓存命中，不发起 HTTP 请求）
        let result2 = provider.lookup("8.8.8.8").await;
        assert!(result2.is_some(), "Second lookup should hit cache");
        assert_eq!(result1, result2, "Cached result should match");
    }

    /// 测试无效 IP 的处理
    /// 依赖外部网络服务，CI 环境可能失败
    #[tokio::test]
    #[ignore]
    async fn test_external_api_provider_invalid_ip() {
        let provider =
            ExternalApiProvider::new("http://ip-api.com/json/{ip}?fields=status,countryCode,city");

        // 私有 IP 查询（ip-api.com 返回 {"status":"fail",...}）
        let result = provider.lookup("192.168.1.1").await;

        // 应该返回 None，因为 API 返回 fail 状态
        assert!(result.is_none(), "Should return None for private IP");
    }

    /// 测试超时处理
    /// 依赖外部网络服务，CI 环境可能失败
    #[test]
    #[ignore]
    fn test_timeout_handling() {
        // 用一个不存在的地址测试超时
        let url = "http://192.0.2.1/timeout-test".to_string(); // TEST-NET, 不可路由

        let result = ExternalApiProvider::fetch_from_api_sync(url);

        // 应该在 2 秒内超时并返回 None
        assert!(result.is_none(), "Should timeout and return None");
    }
}
