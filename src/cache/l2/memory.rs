use crate::cache::traits::L2Cache;
use crate::declare_l2_plugin;
use crate::storages::ShortLink;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;

declare_l2_plugin!("memory", MemoryCache);

#[derive(Default)]
pub struct MemoryCache {
    inner: Arc<DashMap<String, ShortLink>>,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl L2Cache for MemoryCache {
    async fn get(&self, key: &str) -> Option<ShortLink> {
        self.inner.get(key).map(|v| v.value().clone())
    }

    async fn insert(&self, key: String, value: ShortLink) {
        self.inner.insert(key, value);
    }

    async fn remove(&self, key: &str) {
        self.inner.remove(key);
    }

    async fn invalidate_all(&self) {
        self.inner.clear();
    }
}
