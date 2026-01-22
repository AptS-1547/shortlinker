use async_trait::async_trait;
use tracing::trace;

use crate::cache::NegativeCache;

/// 空实现：禁用 Negative Cache
pub struct NullNegativeCache;

impl NullNegativeCache {
    pub fn new() -> Self {
        trace!("NullNegativeCache initialized (negative caching disabled)");
        Self
    }
}

impl Default for NullNegativeCache {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NegativeCache for NullNegativeCache {
    async fn contains(&self, _key: &str) -> bool {
        false
    }

    async fn mark(&self, _key: &str) {}

    async fn remove(&self, _key: &str) {}

    async fn clear(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let cache = NullNegativeCache::new();
        assert!(std::mem::size_of_val(&cache) == 0); // 零大小类型
    }

    #[test]
    fn test_default() {
        let _cache = NullNegativeCache;
    }

    #[tokio::test]
    async fn test_contains_always_returns_false() {
        let cache = NullNegativeCache::new();
        assert!(!cache.contains("any_key").await);
        assert!(!cache.contains("").await);
        assert!(!cache.contains("nonexistent").await);
    }

    #[tokio::test]
    async fn test_mark_is_noop() {
        let cache = NullNegativeCache::new();
        cache.mark("key1").await;
        // mark 后 contains 仍然返回 false
        assert!(!cache.contains("key1").await);
    }

    #[tokio::test]
    async fn test_remove_is_noop() {
        let cache = NullNegativeCache::new();
        cache.remove("key1").await;
        assert!(!cache.contains("key1").await);
    }

    #[tokio::test]
    async fn test_clear_is_noop() {
        let cache = NullNegativeCache::new();
        cache.mark("key1").await;
        cache.clear().await;
        assert!(!cache.contains("key1").await);
    }
}
