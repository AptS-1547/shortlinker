use crate::cache::traits::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;
use crate::storages::ShortLink;
use async_trait::async_trait;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;

declare_object_cache_plugin!("memory", MemoryCache);

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
impl ObjectCache for MemoryCache {
    async fn get(&self, key: &str) -> CacheResult {
        if let Some(value) = self.inner.get(key) {
            CacheResult::Found(value.clone())
        } else {
            CacheResult::NotFound
        }
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

    async fn load_l2_cache(&self, keys: HashMap<String, ShortLink>) {
        for (key, value) in keys {
            self.inner.insert(key, value);
        }
    }
}
