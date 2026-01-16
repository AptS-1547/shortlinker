use crate::cache::negative_cache::MokaNegativeCache;
use crate::cache::register::{get_filter_plugin, get_object_cache_plugin};
use crate::cache::{
    BloomConfig, CacheResult, CompositeCacheTrait, ExistenceFilter, NegativeCache, ObjectCache,
};
use crate::errors::{Result, ShortlinkerError};
use crate::storage::ShortLink;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

pub struct CompositeCache {
    filter_plugin: Arc<dyn ExistenceFilter>,
    object_cache: Arc<dyn ObjectCache>,
    negative_cache: Arc<dyn NegativeCache>,
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

        // 创建 Negative Cache（使用默认配置，后续可扩展为配置项）
        let negative_cache: Arc<dyn NegativeCache> = Arc::new(MokaNegativeCache::new(10000, 60));

        Ok(Arc::new(Self {
            filter_plugin: Arc::from(filter_plugin),
            object_cache: Arc::from(object_cache),
            negative_cache,
        }))
    }
}

#[async_trait]
impl CompositeCacheTrait for CompositeCache {
    async fn get(&self, key: &str) -> CacheResult {
        // Step 1: Bloom Filter 全量加载，false = 确定不存在
        if !self.filter_plugin.check(key).await {
            return CacheResult::NotFound;
        }

        // Step 2: 检查 Negative Cache（数据库确认不存在的 key）
        if self.negative_cache.contains(key).await {
            return CacheResult::NotFound;
        }

        // Step 3: 检查 Object Cache
        self.object_cache.get(key).await
    }

    async fn insert(&self, key: &str, value: ShortLink, ttl_secs: Option<u64>) {
        // 清除 Negative Cache（如果有）
        self.negative_cache.remove(key).await;

        // 写入 Bloom Filter
        self.filter_plugin.set(key).await;

        // 写入 Object Cache
        self.object_cache.insert(key, value, ttl_secs).await;
    }

    async fn remove(&self, key: &str) {
        self.object_cache.remove(key).await;
    }

    async fn mark_not_found(&self, key: &str) {
        self.negative_cache.mark(key).await;
    }

    async fn invalidate_all(&self) {
        self.object_cache.invalidate_all().await;
        self.negative_cache.clear().await;
    }

    async fn load_cache(&self, links: HashMap<String, ShortLink>) {
        self.filter_plugin
            .bulk_set(&links.keys().cloned().collect::<Vec<_>>())
            .await;

        // object_cache 需要先清空再加载
        self.object_cache.invalidate_all().await;
        self.object_cache.load_object_cache(links).await;

        // 清空 negative cache
        self.negative_cache.clear().await;
    }

    async fn load_bloom(&self, codes: &[String]) {
        self.filter_plugin.bulk_set(codes).await;
        // 清空 negative cache（因为 Bloom 重新加载了）
        self.negative_cache.clear().await;
    }

    async fn reconfigure(&self, config: BloomConfig) -> Result<()> {
        self.filter_plugin
            .clear(config.capacity, config.fp_rate)
            .await
    }
}
