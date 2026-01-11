use crate::storage::ShortLink;
use async_trait::async_trait;
use tracing::trace;

use crate::cache::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;

declare_object_cache_plugin!("null", NullObjectCache);

pub struct NullObjectCache;

impl Default for NullObjectCache {
    fn default() -> Self {
        Self::new().expect("NullObjectCache initialization failed")
    }
}

impl NullObjectCache {
    pub fn new() -> Result<Self, String> {
        trace!("Using NullObjectCache: no L2 cache will be used");
        Ok(NullObjectCache)
    }
}

#[async_trait]
impl ObjectCache for NullObjectCache {
    async fn get(&self, key: &str) -> CacheResult {
        trace!("NullObjectCache.get called for key: {}", key);
        CacheResult::NotFound
    }

    async fn insert(&self, key: &str, _: ShortLink, _ttl_secs: Option<u64>) {
        trace!("NullObjectCache.insert called for key: {}", key);
    }

    async fn remove(&self, key: &str) {
        trace!("NullObjectCache.remove called for key: {}", key);
    }

    async fn invalidate_all(&self) {
        trace!("NullObjectCache.invalidate_all called, but no action taken");
    }
}
