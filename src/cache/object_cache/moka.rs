use async_trait::async_trait;
use moka::future::Cache;
use tracing::debug;

use crate::cache::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;
use crate::repository::ShortLink;

declare_object_cache_plugin!("memory", MokaCacheWrapper);

pub struct MokaCacheWrapper {
    inner: Cache<String, ShortLink>,
}

impl Default for MokaCacheWrapper {
    fn default() -> Self {
        Self::new().expect("MokaCacheWrapper initialization failed")
    }
}

impl MokaCacheWrapper {
    pub fn new() -> Result<Self, String> {
        let config = crate::system::app_config::get_config();
        let inner = Cache::builder()
            .max_capacity(config.cache.memory.max_capacity)
            .time_to_live(std::time::Duration::from_secs(config.cache.default_ttl))
            .build();

        debug!(
            "MokaCacheWrapper initialized with max capacity: {}",
            config.cache.memory.max_capacity
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

    async fn insert(&self, key: &str, value: ShortLink) {
        self.inner.insert(key.to_string(), value).await;
    }

    async fn remove(&self, key: &str) {
        self.inner.invalidate(key).await;
    }

    async fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }
}
