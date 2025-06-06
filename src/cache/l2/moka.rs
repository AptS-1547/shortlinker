use async_trait::async_trait;
use moka::future::Cache;

use crate::cache::{CacheResult, L2Cache};
use crate::declare_l2_plugin;
use crate::storages::ShortLink;

declare_l2_plugin!("moka", MokaCacheWrapper);

pub struct MokaCacheWrapper {
    inner: Cache<String, ShortLink>,
}

impl Default for MokaCacheWrapper {
    fn default() -> Self {
        Self::new()
    }
}

impl MokaCacheWrapper {
    pub fn new() -> Self {
        let inner = Cache::builder()
            .max_capacity(10_000)
            .time_to_live(std::time::Duration::from_secs(900))
            .time_to_idle(std::time::Duration::from_secs(300))
            .build();
        Self { inner }
    }
}

#[async_trait]
impl L2Cache for MokaCacheWrapper {
    async fn get(&self, key: &str) -> CacheResult {
        if let Some(value) = self.inner.get(key).await {
            CacheResult::Found(value.clone())
        } else {
            CacheResult::NotFound
        }
    }

    async fn insert(&self, key: String, value: ShortLink) {
        self.inner.insert(key, value).await;
    }

    async fn remove(&self, key: &str) {
        self.inner.invalidate(key).await;
    }

    async fn invalidate_all(&self) {
        self.inner.invalidate_all();
    }
}
