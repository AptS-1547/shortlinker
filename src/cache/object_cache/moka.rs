use async_trait::async_trait;
use moka::future::Cache;
use moka::policy::Expiry;
use std::time::{Duration, Instant};
use tracing::debug;

use crate::cache::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;
use crate::storage::ShortLink;

declare_object_cache_plugin!("memory", MokaCacheWrapper);

/// 自定义过期策略，基于 ShortLink.expires_at 计算过期时间
struct ShortLinkExpiry {
    default_ttl: Duration,
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
                    Some(Duration::from_secs(
                        remaining.min(self.default_ttl.as_secs()),
                    ))
                }
            }
            None => Some(self.default_ttl), // 无过期时间，使用默认 TTL
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
            CacheResult::ExistsButNoValue
        }
    }

    async fn insert(&self, key: &str, value: ShortLink, _ttl_secs: Option<u64>) {
        // 忽略 ttl_secs，Expiry trait 会从 value.expires_at 计算过期时间
        self.inner.insert(key.to_string(), value).await;
    }

    async fn remove(&self, key: &str) {
        self.inner.invalidate(key).await;
    }

    async fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }
}
