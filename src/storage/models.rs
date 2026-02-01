use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortLink {
    pub code: String,
    pub target: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub password: Option<String>,

    #[serde(default)]
    pub click: usize,
}

impl ShortLink {
    /// 检查链接是否已过期
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| exp <= chrono::Utc::now())
            .unwrap_or(false)
    }

    /// 计算缓存 TTL（秒），已过期返回 None
    pub fn cache_ttl(&self, default_ttl: u64) -> Option<u64> {
        match self.expires_at {
            Some(exp) => {
                let now = chrono::Utc::now();
                if exp <= now {
                    None // 已过期，不应缓存
                } else {
                    let remaining = (exp - now).num_seconds() as u64;
                    Some(remaining.min(default_ttl))
                }
            }
            None => Some(default_ttl), // 无过期时间，用默认 TTL
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageConfig {
    pub storage_type: String,
    pub support_click: bool,
}

/// 链接统计信息
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LinkStats {
    pub total_links: usize,
    pub total_clicks: usize,
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
