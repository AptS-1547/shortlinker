use crate::cache::register::{get_filter_plugin, get_object_cache_plugin};
use crate::cache::{BloomConfig, CacheResult, CompositeCacheTrait, ExistenceFilter, ObjectCache};
use crate::errors::{Result, ShortlinkerError};
use crate::storage::ShortLink;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CompositeCache {
    filter_plugin: Arc<dyn ExistenceFilter>,
    object_cache: Arc<dyn ObjectCache>,
}

impl CompositeCache {
    pub async fn create() -> Result<Arc<dyn CompositeCacheTrait>> {
        let config = crate::config::get_config();

        let filter_plugin_name = "bloom";
        let object_cache_name = &config.cache.cache_type;

        let filter_plugin_ctor = get_filter_plugin(filter_plugin_name).ok_or_else(|| {
            ShortlinkerError::cache_plugin_not_found(format!(
                "Filter plugin not found: {}",
                filter_plugin_name
            ))
        })?;
        let object_cache_ctor = get_object_cache_plugin(object_cache_name).ok_or_else(|| {
            ShortlinkerError::cache_plugin_not_found(format!(
                "Object Cache plugin not found: {}",
                object_cache_name
            ))
        })?;

        let filter_plugin = filter_plugin_ctor().await?;
        let object_cache = object_cache_ctor().await?;

        Ok(Arc::new(Self {
            filter_plugin: Arc::from(filter_plugin),
            object_cache: Arc::from(object_cache),
        }))
    }
}

#[async_trait]
impl CompositeCacheTrait for CompositeCache {
    async fn get(&self, key: &str) -> CacheResult {
        if !self.filter_plugin.check(key).await {
            return CacheResult::NotFound;
        }
        return self.object_cache.get(key).await;
    }

    async fn insert(&self, key: &str, value: ShortLink, ttl_secs: Option<u64>) {
        self.filter_plugin.set(key).await;
        self.object_cache.insert(key, value, ttl_secs).await;
    }

    async fn remove(&self, key: &str) {
        self.object_cache.remove(key).await;
    }

    async fn invalidate_all(&self) {
        self.object_cache.invalidate_all().await;
    }

    async fn load_cache(&self, links: HashMap<String, ShortLink>) {
        self.filter_plugin
            .bulk_set(&links.keys().cloned().collect::<Vec<_>>())
            .await;

        // object_cache 需要先清空再加载
        self.object_cache.invalidate_all().await;
        self.object_cache.load_object_cache(links).await;
    }

    async fn reconfigure(&self, config: BloomConfig) -> Result<()> {
        self.filter_plugin
            .clear(config.capacity, config.fp_rate)
            .await
    }
}
