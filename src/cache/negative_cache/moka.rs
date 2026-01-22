use async_trait::async_trait;
use moka::future::Cache;
use std::time::Duration;
use tracing::trace;

use crate::cache::NegativeCache;

pub struct MokaNegativeCache {
    inner: Cache<String, ()>,
}

impl MokaNegativeCache {
    pub fn new(max_capacity: u64, ttl_secs: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();

        trace!(
            "MokaNegativeCache initialized: max_capacity={}, ttl={}s",
            max_capacity, ttl_secs
        );

        Self { inner }
    }
}

#[async_trait]
impl NegativeCache for MokaNegativeCache {
    async fn contains(&self, key: &str) -> bool {
        let result = self.inner.contains_key(key);
        if result {
            trace!("Negative cache hit for key: {}", key);
        }
        result
    }

    async fn mark(&self, key: &str) {
        trace!("Marking key as not found: {}", key);
        self.inner.insert(key.to_string(), ()).await;
    }

    async fn remove(&self, key: &str) {
        trace!("Removing key from negative cache: {}", key);
        self.inner.invalidate(key).await;
    }

    async fn clear(&self) {
        trace!("Clearing negative cache");
        self.inner.invalidate_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mark_and_contains() {
        let cache = MokaNegativeCache::new(1000, 60);

        // 初始状态不包含任何 key
        assert!(!cache.contains("test_key").await);

        // 标记后应该包含
        cache.mark("test_key").await;
        assert!(cache.contains("test_key").await);

        // 其他 key 不受影响
        assert!(!cache.contains("other_key").await);
    }

    #[tokio::test]
    async fn test_remove() {
        let cache = MokaNegativeCache::new(1000, 60);

        cache.mark("test_key").await;
        assert!(cache.contains("test_key").await);

        cache.remove("test_key").await;
        assert!(!cache.contains("test_key").await);
    }

    #[tokio::test]
    async fn test_clear() {
        let cache = MokaNegativeCache::new(1000, 60);

        cache.mark("key1").await;
        cache.mark("key2").await;
        cache.mark("key3").await;

        assert!(cache.contains("key1").await);
        assert!(cache.contains("key2").await);
        assert!(cache.contains("key3").await);

        cache.clear().await;

        // Moka 的 invalidate_all 是异步的，需要等待同步
        cache.inner.run_pending_tasks().await;

        assert!(!cache.contains("key1").await);
        assert!(!cache.contains("key2").await);
        assert!(!cache.contains("key3").await);
    }

    #[tokio::test]
    async fn test_ttl_expiry() {
        // 创建一个 TTL 为 1 秒的缓存
        let cache = MokaNegativeCache::new(1000, 1);

        cache.mark("expiring_key").await;
        assert!(cache.contains("expiring_key").await);

        // 等待 TTL 过期
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;

        // 触发过期清理
        cache.inner.run_pending_tasks().await;

        assert!(!cache.contains("expiring_key").await);
    }
}
