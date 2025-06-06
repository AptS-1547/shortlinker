use crate::storages::ShortLink;
use async_trait::async_trait;
use tracing::debug;

use crate::cache::{CacheResult, L2Cache};
use crate::declare_l2_plugin;

declare_l2_plugin!("null", NullL2Cache);

pub struct NullL2Cache;

impl Default for NullL2Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl NullL2Cache {
    pub fn new() -> Self {
        debug!("Using NullL2Cache: no L2 cache will be used");
        NullL2Cache
    }
}

#[async_trait]
impl L2Cache for NullL2Cache {
    async fn get(&self, key: &str) -> CacheResult {
        debug!("NullL2Cache.get called for key: {}", key);
        CacheResult::NotFound
    }

    async fn insert(&self, key: String, _: ShortLink) {
        debug!("NullL2Cache.insert called for key: {}", key);
    }

    async fn remove(&self, key: &str) {
        debug!("NullL2Cache.remove called for key: {}", key);
    }

    async fn invalidate_all(&self) {
        debug!("NullL2Cache.invalidate_all called, but no action taken");
    }
}
