use crate::cache::register::{get_l1_plugin, get_l2_plugin};
use crate::cache::{BloomConfig, CacheResult, CompositeCacheTrait, ExistenceFilter, ObjectCache};
use crate::errors::ShortlinkerError;
use crate::storages::{CachePreference, ShortLink};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CompositeCache {
    l1: Arc<dyn ExistenceFilter>,
    l2: Arc<dyn ObjectCache>,
}

impl CompositeCache {
    pub async fn create(
        pref: CachePreference,
    ) -> Result<Arc<dyn CompositeCacheTrait>, ShortlinkerError> {
        let l1_ctor = get_l1_plugin(&pref.l1).ok_or_else(|| {
            ShortlinkerError::cache_plugin_not_found(format!("L1 plugin not found: {}", &pref.l1))
        })?;
        let l2_ctor = get_l2_plugin(&pref.l2).ok_or_else(|| {
            ShortlinkerError::cache_plugin_not_found(format!("L2 plugin not found: {}", &pref.l2))
        })?;

        let l1 = l1_ctor().await?;
        let l2 = l2_ctor().await?;

        Ok(Arc::new(Self {
            l1: Arc::from(l1),
            l2: Arc::from(l2),
        }))
    }
}

#[async_trait]
impl CompositeCacheTrait for CompositeCache {
    async fn get(&self, key: &str) -> CacheResult {
        if !self.l1.check(key).await {
            return CacheResult::NotFound;
        }
        return self.l2.get(key).await;
    }

    async fn insert(&self, key: String, value: ShortLink) {
        self.l1.set(&key).await;
        self.l2.insert(key, value).await;
    }

    async fn remove(&self, key: &str) {
        self.l2.remove(key).await;
    }

    async fn invalidate_all(&self) {
        self.l2.invalidate_all().await;
    }

    async fn load_cache(&self, links: HashMap<String, ShortLink>) {
        self.l1
            .bulk_set(&links.keys().cloned().collect::<Vec<_>>())
            .await;

        // L2 需要先清空再加载
        self.l2.invalidate_all().await;
        self.l2.load_l2_cache(links).await;
    }

    async fn reconfigure(&self, config: BloomConfig) {
        self.l1.clear(config.capacity, config.fp_rate).await;
    }
}
