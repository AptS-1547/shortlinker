//! Null (no-op) implementation of CompositeCacheTrait
//!
//! Used by TUI and other contexts that need a LinkService but don't require caching.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use super::traits::{BloomConfig, CacheHealthStatus, CacheResult, CompositeCacheTrait};
use crate::errors::Result;
use crate::storage::ShortLink;

/// A no-op composite cache that does nothing.
///
/// All reads return `CacheResult::Miss`, all writes are ignored.
/// `bloom_check` returns `true` (conservative: "might exist") to avoid
/// false negatives that would skip database lookups.
pub struct NullCompositeCache;

impl NullCompositeCache {
    pub fn arc() -> Arc<dyn CompositeCacheTrait> {
        Arc::new(Self)
    }
}

#[async_trait]
impl CompositeCacheTrait for NullCompositeCache {
    async fn get(&self, _key: &str) -> CacheResult {
        CacheResult::Miss
    }
    async fn insert(&self, _key: &str, _value: ShortLink, _ttl_secs: Option<u64>) {}
    async fn remove(&self, _key: &str) {}
    async fn invalidate_all(&self) {}
    async fn rebuild_all(&self) -> Result<()> {
        Ok(())
    }
    async fn mark_not_found(&self, _key: &str) {}
    async fn load_cache(&self, _links: HashMap<String, ShortLink>) {}
    async fn load_bloom(&self, _codes: &[String]) {}
    async fn reconfigure(&self, _config: BloomConfig) -> Result<()> {
        Ok(())
    }
    async fn bloom_check(&self, _key: &str) -> bool {
        true
    }
    async fn health_check(&self) -> CacheHealthStatus {
        CacheHealthStatus {
            status: "disabled".to_string(),
            cache_type: "null".to_string(),
            bloom_filter_enabled: false,
            negative_cache_enabled: false,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_null_cache_get_returns_miss() {
        let cache = NullCompositeCache;
        assert!(matches!(cache.get("any-key").await, CacheResult::Miss));
    }

    #[tokio::test]
    async fn test_null_cache_bloom_check_returns_true() {
        let cache = NullCompositeCache;
        assert!(cache.bloom_check("any-key").await);
    }

    #[tokio::test]
    async fn test_null_cache_health_check() {
        let cache = NullCompositeCache;
        let health = cache.health_check().await;
        assert_eq!(health.status, "disabled");
        assert_eq!(health.cache_type, "null");
    }

    #[tokio::test]
    async fn test_null_cache_operations_dont_panic() {
        let cache = NullCompositeCache;
        let link = ShortLink {
            code: "test".to_string(),
            target: "https://example.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            password: None,
            click: 0,
        };
        cache.insert("test", link.clone(), Some(60)).await;
        cache.remove("test").await;
        cache.invalidate_all().await;
        cache.rebuild_all().await.unwrap();
        cache.mark_not_found("test").await;
        cache
            .load_cache(HashMap::from([("test".to_string(), link)]))
            .await;
        cache.load_bloom(&["test".to_string()]).await;
        cache
            .reconfigure(BloomConfig {
                capacity: 100,
                fp_rate: 0.01,
            })
            .await
            .unwrap();
    }
}
