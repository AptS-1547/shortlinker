use crate::storage::ShortLink;
use async_trait::async_trait;
use tracing::trace;

use crate::cache::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;

declare_object_cache_plugin!("null", NullObjectCache);

pub struct NullObjectCache;

impl NullObjectCache {
    pub async fn new() -> Result<Self, String> {
        trace!("Using NullObjectCache: no L2 cache will be used");
        Ok(NullObjectCache)
    }
}

#[async_trait]
impl ObjectCache for NullObjectCache {
    async fn get(&self, key: &str) -> CacheResult {
        trace!("NullObjectCache.get called for key: {}", key);
        CacheResult::Miss
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

#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn test_null_cache_new() {
        let cache = NullObjectCache::new().await;
        assert!(cache.is_ok());
    }

    #[tokio::test]
    async fn test_null_cache_get_always_returns_miss() {
        let cache = NullObjectCache::new().await.unwrap();

        // 任何 key 都应该返回 Miss
        let result = cache.get("any_key").await;
        assert!(matches!(result, CacheResult::Miss));

        let result = cache.get("another_key").await;
        assert!(matches!(result, CacheResult::Miss));

        let result = cache.get("").await;
        assert!(matches!(result, CacheResult::Miss));
    }

    #[tokio::test]
    async fn test_null_cache_insert_is_noop() {
        let cache = NullObjectCache::new().await.unwrap();
        let link = create_test_link("test");

        // insert 应该是空操作，不会 panic
        cache.insert("test", link.clone(), None).await;
        cache.insert("test", link.clone(), Some(3600)).await;

        // 插入后 get 仍然返回 Miss
        let result = cache.get("test").await;
        assert!(matches!(result, CacheResult::Miss));
    }

    #[tokio::test]
    async fn test_null_cache_remove_is_noop() {
        let cache = NullObjectCache::new().await.unwrap();

        // remove 应该是空操作，不会 panic
        cache.remove("nonexistent").await;
        cache.remove("").await;
    }

    #[tokio::test]
    async fn test_null_cache_invalidate_all_is_noop() {
        let cache = NullObjectCache::new().await.unwrap();

        // invalidate_all 应该是空操作，不会 panic
        cache.invalidate_all().await;
    }
}
