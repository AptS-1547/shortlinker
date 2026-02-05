use crate::cache::negative_cache::MokaNegativeCache;
use crate::cache::register::{get_filter_plugin, get_object_cache_plugin};
use crate::cache::{
    BloomConfig, CacheHealthStatus, CacheResult, CompositeCacheTrait, ExistenceFilter,
    NegativeCache, ObjectCache,
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
        // Bloom Filter 不支持删除，用 Negative Cache 拦截后续请求
        self.negative_cache.mark(key).await;
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

        // ObjectCache 不预热，走 cache-aside 按需回填
        // 旧数据会在 TTL 到期后自然过期

        // 清空 negative cache（因为 Bloom 重新加载了，之前的 not_found 状态可能已失效）
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

    async fn bloom_check(&self, key: &str) -> bool {
        self.filter_plugin.check(key).await
    }

    async fn health_check(&self) -> CacheHealthStatus {
        let config = crate::config::get_config();
        let cache_type = config.cache.cache_type.clone();

        // Bloom filter 和 Negative cache 在创建时就初始化了，如果能到这里就是健康的
        CacheHealthStatus {
            status: "healthy".to_string(),
            cache_type,
            bloom_filter_enabled: true,
            negative_cache_enabled: true,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::existence_filter::bloom::BloomExistenceFilterPlugin;
    use crate::cache::object_cache::null::NullObjectCache;

    fn create_test_link(code: &str) -> ShortLink {
        ShortLink {
            code: code.to_string(),
            target: "https://example.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            password: None,
            click: 0,
        }
    }

    /// 创建测试用的 CompositeCache（不依赖全局配置）
    async fn create_test_composite() -> CompositeCache {
        let filter = BloomExistenceFilterPlugin::new().unwrap();
        let object_cache = NullObjectCache::new().await.unwrap();
        let negative_cache = MokaNegativeCache::new(1000, 60);

        CompositeCache {
            filter_plugin: Arc::new(filter),
            object_cache: Arc::new(object_cache),
            negative_cache: Arc::new(negative_cache),
        }
    }

    #[tokio::test]
    async fn test_composite_get_not_in_bloom_returns_not_found() {
        let cache = create_test_composite().await;

        // key 不在 Bloom Filter 中，应该返回 NotFound
        let result = cache.get("nonexistent_key").await;
        assert!(matches!(result, CacheResult::NotFound));
    }

    #[tokio::test]
    async fn test_composite_get_in_bloom_but_not_cached_returns_miss() {
        let cache = create_test_composite().await;

        // 先将 key 加入 Bloom Filter
        cache.filter_plugin.set("test_key").await;

        // key 在 Bloom 中但不在 Object Cache 中（NullObjectCache 总是返回 Miss）
        let result = cache.get("test_key").await;
        assert!(matches!(result, CacheResult::Miss));
    }

    #[tokio::test]
    async fn test_composite_get_in_negative_cache_returns_not_found() {
        let cache = create_test_composite().await;

        // 先将 key 加入 Bloom Filter
        cache.filter_plugin.set("negative_key").await;

        // 然后标记为 not found
        cache.mark_not_found("negative_key").await;

        // 应该返回 NotFound（被 Negative Cache 拦截）
        let result = cache.get("negative_key").await;
        assert!(matches!(result, CacheResult::NotFound));
    }

    #[tokio::test]
    async fn test_composite_insert_clears_negative_cache() {
        let cache = create_test_composite().await;
        let link = create_test_link("insert_test");

        // 先标记为 not found
        cache.mark_not_found("insert_test").await;
        assert!(cache.negative_cache.contains("insert_test").await);

        // 插入数据
        cache.insert("insert_test", link, None).await;

        // Negative Cache 应该被清除
        assert!(!cache.negative_cache.contains("insert_test").await);

        // Bloom Filter 应该包含这个 key
        assert!(cache.filter_plugin.check("insert_test").await);
    }

    #[tokio::test]
    async fn test_composite_remove_marks_negative_cache() {
        let cache = create_test_composite().await;
        let link = create_test_link("remove_test");

        // 先插入
        cache.insert("remove_test", link, None).await;

        // 删除
        cache.remove("remove_test").await;

        // 应该被标记到 Negative Cache
        assert!(cache.negative_cache.contains("remove_test").await);
    }

    #[tokio::test]
    async fn test_composite_invalidate_all() {
        let cache = create_test_composite().await;

        // 添加一些数据
        cache.mark_not_found("key1").await;
        cache.mark_not_found("key2").await;

        // 清空
        cache.invalidate_all().await;

        // Negative Cache 应该被清空
        assert!(!cache.negative_cache.contains("key1").await);
        assert!(!cache.negative_cache.contains("key2").await);
    }

    #[tokio::test]
    async fn test_composite_load_cache() {
        let cache = create_test_composite().await;

        // 先标记一些 key 为 not found
        cache.mark_not_found("load_key1").await;

        // 加载新数据
        let mut links = HashMap::new();
        links.insert("load_key1".to_string(), create_test_link("load_key1"));
        links.insert("load_key2".to_string(), create_test_link("load_key2"));

        cache.load_cache(links).await;

        // Bloom Filter 应该包含这些 key
        assert!(cache.filter_plugin.check("load_key1").await);
        assert!(cache.filter_plugin.check("load_key2").await);

        // Negative Cache 应该被清空
        assert!(!cache.negative_cache.contains("load_key1").await);
    }

    #[tokio::test]
    async fn test_composite_load_bloom() {
        let cache = create_test_composite().await;

        // 先标记一些 key 为 not found
        cache.mark_not_found("bloom_key1").await;

        // 只加载 Bloom
        let codes = vec!["bloom_key1".to_string(), "bloom_key2".to_string()];
        cache.load_bloom(&codes).await;

        // Bloom Filter 应该包含这些 key
        assert!(cache.filter_plugin.check("bloom_key1").await);
        assert!(cache.filter_plugin.check("bloom_key2").await);

        // Negative Cache 应该被清空
        assert!(!cache.negative_cache.contains("bloom_key1").await);
    }

    #[tokio::test]
    async fn test_composite_reconfigure() {
        let cache = create_test_composite().await;

        // 添加一些数据
        cache.filter_plugin.set("reconfig_key").await;

        // 重新配置
        let config = BloomConfig {
            capacity: 5000,
            fp_rate: 0.01,
        };
        let result = cache.reconfigure(config).await;
        assert!(result.is_ok());

        // 重新配置后，之前的数据应该被清空
        // （Bloom Filter 的 clear 会重建）
    }
}
