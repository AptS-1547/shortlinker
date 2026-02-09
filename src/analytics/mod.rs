pub mod global;
pub mod hourly_writer;
pub mod manager;
pub mod retention;
pub mod rollup;
pub mod sink;

pub use hourly_writer::HourlyRollupWriter;
pub use manager::ClickManager;
pub use retention::DataRetentionTask;
pub use rollup::{ClickAggregation, RollupManager, aggregate_click_details};
pub use sink::{ClickSink, DetailedClickSink};

use std::collections::HashMap;

use chrono::{DateTime, Timelike, Utc};
use tracing::warn;

// ============ 公共工具函数 ============

/// 将时间戳截断到整点
pub(crate) fn truncate_to_hour(ts: DateTime<Utc>) -> DateTime<Utc> {
    ts.date_naive()
        .and_hms_opt(ts.time().hour(), 0, 0)
        .unwrap()
        .and_utc()
}

/// 从 JSON 字符串解析计数 HashMap
pub(crate) fn parse_json_counts(json_str: &Option<String>) -> HashMap<String, usize> {
    match json_str {
        Some(s) if !s.is_empty() => serde_json::from_str(s).unwrap_or_else(|e| {
            warn!(
                "Failed to parse JSON counts: {} (data: {})",
                e,
                &s[..s.len().min(200)]
            );
            HashMap::new()
        }),
        _ => HashMap::new(),
    }
}

/// 将计数 HashMap 序列化为 JSON 字符串
pub(crate) fn to_json_string(map: &HashMap<String, usize>) -> String {
    serde_json::to_string(map).unwrap_or_else(|_| "{}".to_string())
}

/// 原始点击事件（用于 channel 传输，避免在热路径做计算）
#[derive(Debug)]
pub struct RawClickEvent {
    /// 短链接代码
    pub code: String,
    /// 原始 query string
    pub query: Option<String>,
    /// Referer header
    pub referrer: Option<String>,
    /// User-Agent header
    pub user_agent: Option<String>,
    /// 客户端 IP
    pub ip: Option<String>,
}

/// 详细点击信息
#[derive(Debug, Clone)]
pub struct ClickDetail {
    /// 短链接代码
    pub code: String,
    /// 点击时间戳
    pub timestamp: DateTime<Utc>,
    /// 来源页面 (Referer header)
    pub referrer: Option<String>,
    /// UserAgent hash (references user_agents.hash)
    pub user_agent_hash: Option<String>,
    /// 客户端 IP 地址
    pub ip_address: Option<String>,
    /// 国家代码 (ISO 3166-1 alpha-2)
    pub country: Option<String>,
    /// 城市名称
    pub city: Option<String>,
    /// 流量来源 (utm_source 参数值, ref:{domain}, 或 direct)
    pub source: Option<String>,
}

impl ClickDetail {
    /// 创建新的点击详情
    pub fn new(code: String) -> Self {
        Self {
            code,
            timestamp: Utc::now(),
            referrer: None,
            user_agent_hash: None,
            ip_address: None,
            country: None,
            city: None,
            source: None,
        }
    }

    /// 设置地理位置信息
    pub fn with_geo(mut self, country: Option<String>, city: Option<String>) -> Self {
        self.country = country;
        self.city = city;
        self
    }
}
