use crate::storages::ShortLink;
use async_trait::async_trait;
use tracing::debug;

use crate::cache::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;

declare_object_cache_plugin!("null", NullObjectCache);

pub struct NullObjectCache;

impl Default for NullObjectCache {
    fn default() -> Self {
        Self::new()
    }
}

impl NullObjectCache {
    pub fn new() -> Self {
        debug!("Using NullObjectCache: no L2 cache will be used");
        NullObjectCache
    }
}

#[async_trait]
impl ObjectCache for NullObjectCache {
    async fn get(&self, key: &str) -> CacheResult {
        debug!("NullObjectCache.get called for key: {}", key);
        CacheResult::NotFound
    }

    async fn insert(&self, key: String, _: ShortLink) {
        debug!("NullObjectCache.insert called for key: {}", key);
    }

    async fn remove(&self, key: &str) {
        debug!("NullObjectCache.remove called for key: {}", key);
    }

    async fn invalidate_all(&self) {
        debug!("NullObjectCache.invalidate_all called, but no action taken");
    }
}
