//! GeoIP 服务模块
//!
//! 提供 IP 地址地理位置查询功能，支持：
//! - MaxMind GeoLite2 本地数据库
//! - 外部 API fallback (ip-api.com)

mod external_api;
mod maxmind;
mod provider;

pub use provider::{GeoInfo, GeoIpLookup, GeoIpProvider};
