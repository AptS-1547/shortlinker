//! 外部 GeoIP API 实现
//!
//! 使用外部 HTTP API 进行 IP 地理位置查询（如 ip-api.com）

use std::time::Duration;

use async_trait::async_trait;
use tracing::{trace, warn};

use super::provider::{GeoInfo, GeoIpLookup};

/// 外部 API GeoIP Provider
pub struct ExternalApiProvider {
    client: reqwest::Client,
    api_url_template: String,
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
            .unwrap_or_default();

        Self {
            client,
            api_url_template: api_url_template.to_string(),
        }
    }
}

#[async_trait]
impl GeoIpLookup for ExternalApiProvider {
    async fn lookup(&self, ip: &str) -> Option<GeoInfo> {
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

    fn name(&self) -> &'static str {
        "ExternalAPI"
    }
}
