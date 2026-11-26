use serde::{Deserialize, Serialize};

/// 短链接的领域模型，由 storage 层在各后端模型之间统一转换。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLink {
    /// 短码，全局唯一，即 URL 路径中的标识部分。
    pub code: String,
    /// 重定向目标 URL，完整形式（含 scheme）。
    pub target: String,
    /// 创建时间（UTC）。
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 过期时间（UTC）；`None` 表示永不过期。
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// 访问口令的哈希值；`None` 表示链接无密码保护。
    pub password: Option<String>,

    /// 累计点击次数，反序列化缺失时默认 0。
    #[serde(default)]
    pub click: usize,
}

impl ShortLink {
    /// 检查链接是否已过期
    pub fn is_expired(&self) -> bool {
        self.expires_at.is_some_and(|exp| exp <= chrono::Utc::now())
    }

    /// 计算缓存 TTL（秒），已过期返回 None
    pub fn cache_ttl(&self, default_ttl: u64) -> Option<u64> {
        match self.expires_at {
            Some(exp) => {
                let now = chrono::Utc::now();
                if exp <= now {
                    None // 已过期，不应缓存
                } else {
                    let remaining = (exp - now).num_seconds().max(0).cast_unsigned();
                    Some(remaining.min(default_ttl))
                }
            }
            None => Some(default_ttl), // 无过期时间，用默认 TTL
        }
    }
}

/// 存储后端的能力描述，供健康检查等接口展示。
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageConfig {
    /// 后端标识名称（如 "sqlite"、"mysql"、"postgresql"）。
    pub storage_type: String,
    /// 后端是否支持点击计数写入。
    pub support_click: bool,
}

/// 链接统计信息
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LinkStats {
    /// 全部短链接数量（含已过期）。
    pub total_links: usize,
    /// 全部短链接的点击总数。
    pub total_clicks: usize,
    /// 尚未过期的短链接数量。
    pub active_links: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_test_link(expires_at: Option<chrono::DateTime<Utc>>) -> ShortLink {
        ShortLink {
            code: "test".to_string(),
            target: "https://example.com".to_string(),
            created_at: Utc::now(),
            expires_at,
            password: None,
            click: 0,
        }
    }

    #[test]
    fn test_is_expired_no_expiration() {
        let link = create_test_link(None);
        assert!(!link.is_expired());
    }

    #[test]
    fn test_is_expired_future_expiration() {
        let future = Utc::now() + Duration::hours(1);
        let link = create_test_link(Some(future));
        assert!(!link.is_expired());
    }

    #[test]
    fn test_is_expired_past_expiration() {
        let past = Utc::now() - Duration::hours(1);
        let link = create_test_link(Some(past));
        assert!(link.is_expired());
    }

    #[test]
    fn test_is_expired_exact_now() {
        // 边界情况：刚好等于当前时间应该算过期
        let now = Utc::now();
        let link = create_test_link(Some(now));
        assert!(link.is_expired());
    }

    #[test]
    fn test_cache_ttl_no_expiration() {
        let link = create_test_link(None);
        let ttl = link.cache_ttl(3600);
        assert_eq!(ttl, Some(3600));
    }

    #[test]
    fn test_cache_ttl_expired_returns_none() {
        let past = Utc::now() - Duration::hours(1);
        let link = create_test_link(Some(past));
        let ttl = link.cache_ttl(3600);
        assert_eq!(ttl, None);
    }

    #[test]
    fn test_cache_ttl_uses_remaining_time() {
        let future = Utc::now() + Duration::seconds(100);
        let link = create_test_link(Some(future));
        let ttl = link.cache_ttl(3600);
        // 剩余时间约 100 秒，应该小于默认 TTL
        assert!(ttl.is_some());
        let ttl_val = ttl.unwrap();
        assert!(ttl_val <= 100);
        assert!(ttl_val >= 98); // 允许少量时间误差
    }

    #[test]
    fn test_cache_ttl_caps_at_default() {
        // 过期时间远在未来，应该使用默认 TTL
        let future = Utc::now() + Duration::days(365);
        let link = create_test_link(Some(future));
        let ttl = link.cache_ttl(3600);
        assert_eq!(ttl, Some(3600));
    }

    #[test]
    fn test_link_stats_default() {
        let stats = LinkStats::default();
        assert_eq!(stats.total_links, 0);
        assert_eq!(stats.total_clicks, 0);
        assert_eq!(stats.active_links, 0);
    }
}
