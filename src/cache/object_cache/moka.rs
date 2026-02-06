use async_trait::async_trait;
use moka::future::Cache;
use moka::policy::Expiry;
use rand::Rng;
use std::time::{Duration, Instant};
use tracing::debug;

use crate::cache::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;
use crate::storage::ShortLink;

declare_object_cache_plugin!("memory", MokaCacheWrapper);

/// 自定义过期策略，基于 ShortLink.expires_at 计算过期时间
/// 添加 ±10% 随机抖动避免缓存集中失效
struct ShortLinkExpiry {
    default_ttl: Duration,
}

impl ShortLinkExpiry {
    /// 添加 ±10% 随机抖动到 TTL
    fn apply_jitter(ttl_secs: u64) -> u64 {
        if ttl_secs == 0 {
            return 0;
        }
        let jitter_range = (ttl_secs / 10).max(1); // 至少 1 秒抖动
        let jitter = rand::rng().random_range(0..=jitter_range * 2);
        // 范围: ttl - 10% 到 ttl + 10%
        ttl_secs.saturating_sub(jitter_range).saturating_add(jitter)
    }
}

impl Expiry<String, ShortLink> for ShortLinkExpiry {
    fn expire_after_create(
        &self,
        _key: &String,
        value: &ShortLink,
        _created_at: Instant,
    ) -> Option<Duration> {
        match value.expires_at {
            Some(expires_at) => {
                let now = chrono::Utc::now();
                if expires_at <= now {
                    // 已过期，设置极短 TTL
                    Some(Duration::from_secs(1))
                } else {
                    let remaining = (expires_at - now).num_seconds() as u64;
                    let capped = remaining.min(self.default_ttl.as_secs());
                    // 添加随机抖动避免缓存雪崩
                    Some(Duration::from_secs(Self::apply_jitter(capped)))
                }
            }
            None => {
                // 无过期时间，使用默认 TTL 并添加抖动
                Some(Duration::from_secs(Self::apply_jitter(
                    self.default_ttl.as_secs(),
                )))
            }
        }
    }
}

pub struct MokaCacheWrapper {
    inner: Cache<String, ShortLink>,
}

impl MokaCacheWrapper {
    pub async fn new() -> Result<Self, String> {
        let config = crate::config::get_config();
        let default_ttl = Duration::from_secs(config.cache.default_ttl);

        let inner = Cache::builder()
            .max_capacity(config.cache.memory.max_capacity)
            .expire_after(ShortLinkExpiry { default_ttl })
            .build();

        debug!(
            "MokaCacheWrapper initialized with max capacity: {}, default TTL: {}s",
            config.cache.memory.max_capacity, config.cache.default_ttl
        );
        Ok(Self { inner })
    }
}

#[async_trait]
impl ObjectCache for MokaCacheWrapper {
    async fn get(&self, key: &str) -> CacheResult {
        if let Some(value) = self.inner.get(key).await {
            CacheResult::Found(value.clone())
        } else {
            CacheResult::Miss
        }
    }

    async fn insert(&self, key: &str, value: ShortLink, _ttl_secs: Option<u64>) {
        // 忽略 ttl_secs，Expiry trait 会从 value.expires_at 计算过期时间
        self.inner.insert(key.to_string(), value).await;
    }

    async fn remove(&self, key: &str) {
        self.inner.invalidate(key).await;
    }

    fn entry_count(&self) -> u64 {
        self.inner.entry_count()
    }

    /// Invalidates all entries in the cache.
    ///
    /// # Note
    /// Moka's `invalidate_all()` is **lazy** - entries are marked for deletion but
    /// may still be readable for a brief window (typically <1ms). This is by design
    /// for performance. The actual cleanup happens asynchronously via background tasks.
    ///
    /// If strong consistency is required (e.g., in tests), call `run_pending_tasks()`
    /// after `invalidate_all()`, but be aware this has a performance cost.
    async fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    fn create_test_link(expires_at: Option<chrono::DateTime<chrono::Utc>>) -> ShortLink {
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
    fn test_expiry_no_expiration_uses_default_with_jitter() {
        let expiry = ShortLinkExpiry {
            default_ttl: std::time::Duration::from_secs(3600),
        };
        let link = create_test_link(None);

        let result = expiry.expire_after_create(&"key".to_string(), &link, Instant::now());

        // 应该是 3600 ± 10% (3240 - 3960)
        assert!(result.is_some());
        let ttl = result.unwrap().as_secs();
        assert!(ttl >= 3240 && ttl <= 3960, "TTL {} not in expected range", ttl);
    }

    #[test]
    fn test_expiry_already_expired_returns_short_ttl() {
        let expiry = ShortLinkExpiry {
            default_ttl: std::time::Duration::from_secs(3600),
        };
        let past = Utc::now() - Duration::hours(1);
        let link = create_test_link(Some(past));

        let result = expiry.expire_after_create(&"key".to_string(), &link, Instant::now());

        // 已过期应该返回 1 秒的极短 TTL
        assert_eq!(result, Some(std::time::Duration::from_secs(1)));
    }

    #[test]
    fn test_expiry_future_uses_remaining_time_with_jitter() {
        let expiry = ShortLinkExpiry {
            default_ttl: std::time::Duration::from_secs(3600),
        };
        let future = Utc::now() + Duration::seconds(100);
        let link = create_test_link(Some(future));

        let result = expiry.expire_after_create(&"key".to_string(), &link, Instant::now());

        // 剩余时间约 100 秒 ± 10% (90 - 110)
        assert!(result.is_some());
        let ttl = result.unwrap().as_secs();
        assert!(ttl >= 88 && ttl <= 112, "TTL {} not in expected range", ttl);
    }

    #[test]
    fn test_expiry_caps_at_default_ttl_with_jitter() {
        let expiry = ShortLinkExpiry {
            default_ttl: std::time::Duration::from_secs(3600),
        };
        // 过期时间远在未来（1年后）
        let future = Utc::now() + Duration::days(365);
        let link = create_test_link(Some(future));

        let result = expiry.expire_after_create(&"key".to_string(), &link, Instant::now());

        // 应该被限制在默认 TTL ± 10% (3240 - 3960)
        assert!(result.is_some());
        let ttl = result.unwrap().as_secs();
        assert!(ttl >= 3240 && ttl <= 3960, "TTL {} not in expected range", ttl);
    }

    #[test]
    fn test_expiry_exact_boundary() {
        let expiry = ShortLinkExpiry {
            default_ttl: std::time::Duration::from_secs(3600),
        };
        // 刚好等于当前时间
        let now = Utc::now();
        let link = create_test_link(Some(now));

        let result = expiry.expire_after_create(&"key".to_string(), &link, Instant::now());

        // 刚好过期，应该返回 1 秒
        assert_eq!(result, Some(std::time::Duration::from_secs(1)));
    }
}
