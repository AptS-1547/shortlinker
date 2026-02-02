//! MaxMind GeoLite2 数据库实现
//!
//! 使用本地 MaxMind GeoLite2-City.mmdb 文件进行 IP 地理位置查询

use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;
use maxminddb::Reader;
use tracing::trace;

use super::provider::{GeoInfo, GeoIpLookup};

/// MaxMind GeoIP Provider
pub struct MaxMindProvider {
    reader: Arc<Reader<Vec<u8>>>,
}

impl MaxMindProvider {
    /// 从文件路径创建 MaxMind Provider
    pub fn new(path: &str) -> Result<Self, maxminddb::MaxMindDBError> {
        let reader = Reader::open_readfile(path)?;
        Ok(Self {
            reader: Arc::new(reader),
        })
    }
}

#[async_trait]
impl GeoIpLookup for MaxMindProvider {
    async fn lookup(&self, ip: &str) -> Option<GeoInfo> {
        let ip_addr: IpAddr = ip.parse().ok()?;

        let city: maxminddb::geoip2::City = self.reader.lookup(ip_addr).ok()?;

        let country = city
            .country
            .as_ref()
            .and_then(|c| c.iso_code)
            .map(String::from);

        let city_name = city
            .city
            .as_ref()
            .and_then(|c| c.names.as_ref())
            .and_then(|names| names.get("en"))
            .map(|s| (*s).to_string());

        trace!(
            "MaxMind lookup for {}: country={:?}, city={:?}",
            ip, country, city_name
        );

        Some(GeoInfo {
            country,
            city: city_name,
        })
    }

    fn name(&self) -> &'static str {
        "MaxMind"
    }
}
