pub mod global;
pub mod manager;
pub mod sink;

pub use manager::ClickManager;
pub use sink::{ClickSink, DetailedClickSink};

use chrono::{DateTime, Utc};

/// 详细点击信息
#[derive(Debug, Clone)]
pub struct ClickDetail {
    /// 短链接代码
    pub code: String,
    /// 点击时间戳
    pub timestamp: DateTime<Utc>,
    /// 来源页面 (Referer header)
    pub referrer: Option<String>,
    /// 用户代理 (User-Agent header)
    pub user_agent: Option<String>,
    /// 客户端 IP 地址
    pub ip_address: Option<String>,
    /// 国家代码 (ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// 城市名称
    pub city: Option<String>,
}

impl ClickDetail {
    /// 创建新的点击详情
    pub fn new(code: String) -> Self {
        Self {
            code,
            timestamp: Utc::now(),
            referrer: None,
            user_agent: None,
            ip_address: None,
            country: None,
            city: None,
        }
    }

    /// 设置地理位置信息
    pub fn with_geo(mut self, country: Option<String>, city: Option<String>) -> Self {
        self.country = country;
        self.city = city;
        self
    }
}
