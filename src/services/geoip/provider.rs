//! GeoIP Provider 抽象层
//!
//! 统一的 GeoIP 查询接口，根据配置自动选择实现：
//! 1. 检查 maxminddb_path 是否配置且文件可读
//! 2. 可读 → MaxMindProvider
//! 3. 不可读 → ExternalApiProvider

use std::sync::Arc;

use async_trait::async_trait;
use tracing::{debug, info, warn};

use super::external_api::ExternalApiProvider;
use super::maxmind::MaxMindProvider;
use crate::config::AnalyticsConfig;

/// 地理位置信息
#[derive(Debug, Clone, Default)]
pub struct GeoInfo {
    /// ISO 3166-1 alpha-2 国家代码 (e.g., "CN", "US")
    pub country: Option<String>,
    /// 城市名称
    pub city: Option<String>,
}

/// GeoIP 查询 trait
#[async_trait]
pub trait GeoIpLookup: Send + Sync {
    /// 查询 IP 地址的地理位置
    async fn lookup(&self, ip: &str) -> Option<GeoInfo>;

    /// 获取 provider 名称（用于日志）
    fn name(&self) -> &'static str;
}

/// 统一 GeoIP Provider
///
/// 启动时根据配置自动选择实现
pub struct GeoIpProvider {
    inner: Arc<dyn GeoIpLookup>,
}

impl GeoIpProvider {
    /// 根据 AnalyticsConfig 初始化
    ///
    /// 1. 检查 maxminddb_path 是否配置且文件可读
    /// 2. 可读 → MaxMindProvider
    /// 3. 不可读 → ExternalApiProvider
    pub fn new(config: &AnalyticsConfig) -> Self {
        let inner: Arc<dyn GeoIpLookup> = if let Some(ref path) = config.maxminddb_path {
            match MaxMindProvider::new(path) {
                Ok(provider) => {
                    info!("GeoIP: Using MaxMind database at {}", path);
                    Arc::new(provider)
                }
                Err(e) => {
                    warn!(
                        "GeoIP: Failed to load MaxMind database at {}: {}, falling back to external API",
                        path, e
                    );
                    Arc::new(ExternalApiProvider::new(&config.geoip_api_url))
                }
            }
        } else {
            debug!("GeoIP: No MaxMind database configured, using external API");
            Arc::new(ExternalApiProvider::new(&config.geoip_api_url))
        };

        info!("GeoIP: Initialized with {} provider", inner.name());
        Self { inner }
    }

    /// 查询 IP 地址的地理位置
    pub async fn lookup(&self, ip: &str) -> Option<GeoInfo> {
        self.inner.lookup(ip).await
    }

    /// 获取当前使用的 provider 名称
    pub fn provider_name(&self) -> &'static str {
        self.inner.name()
    }
}

impl Clone for GeoIpProvider {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}
