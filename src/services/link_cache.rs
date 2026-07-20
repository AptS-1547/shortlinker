//! Short-link cache policy built directly on AsterForge cache primitives.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use futures_util::StreamExt;

use crate::errors::{Result, ShortlinkerError};
use crate::metrics::MetricsRecorder;
use crate::storage::{SeaOrmStorage, ShortLink};

const INITIAL_BLOOM_CAPACITY: usize = 100;
const BLOOM_FALSE_POSITIVE_RATE: f64 = 0.001;
const NEGATIVE_CACHE_PREFIX: &str = "shortlink:negative:";
const NEGATIVE_CACHE_TTL_SECS: u64 = 60;
const BLOOM_REBUILD_BATCH_SIZE: u64 = 10_000;

/// Result of looking up a short link through the cache query chain.
#[derive(Debug, Clone)]
pub enum LinkCacheLookup {
    /// Bloom or negative cache proves the short code does not exist.
    NotFound,
    /// The code may exist but has no cached object payload.
    Miss,
    /// A valid cached short link was found.
    Found(ShortLink),
}

/// Cache health data presented by the shortlinker health endpoint.
#[derive(Debug, Clone)]
pub struct LinkCacheHealth {
    pub status: String,
    pub cache_type: String,
    pub bloom_filter_enabled: bool,
    pub negative_cache_enabled: bool,
    pub error: Option<String>,
}

/// Shortlinker-owned cache policy shared by redirect and link-management paths.
#[async_trait]
pub trait LinkCache: Send + Sync {
    async fn get(&self, key: &str) -> LinkCacheLookup;
    async fn insert(&self, key: &str, value: ShortLink, ttl_secs: Option<u64>);
    async fn remove(&self, key: &str);
    async fn invalidate_all(&self);
    async fn rebuild_all(&self) -> Result<()>;
    async fn mark_not_found(&self, key: &str);
    async fn bloom_check(&self, key: &str) -> bool;
    async fn health_check(&self) -> LinkCacheHealth;
}

/// Production cache policy using Forge object, negative, and Bloom primitives.
pub struct ForgeLinkCache {
    bloom: Arc<aster_forge_cache::bloom::BloomFilter>,
    objects: Arc<dyn aster_forge_cache::CacheBackend>,
    negatives: Arc<dyn aster_forge_cache::CacheBackend>,
    object_prefix: String,
    metrics: Arc<dyn MetricsRecorder>,
    storage: Arc<SeaOrmStorage>,
}

impl ForgeLinkCache {
    /// Creates the configured Forge backends and the shortlinker cache policy.
    pub async fn create(
        metrics: Arc<dyn MetricsRecorder>,
        storage: Arc<SeaOrmStorage>,
    ) -> Result<Arc<dyn LinkCache>> {
        let config = crate::config::get_config();
        let objects = aster_forge_cache::create_cache(&aster_forge_cache::CacheConfig {
            backend: config.cache.cache_type.clone(),
            endpoint: config.cache.redis.url.clone(),
            default_ttl: config.cache.default_ttl,
        })
        .await;
        let negatives: Arc<dyn aster_forge_cache::CacheBackend> =
            Arc::new(aster_forge_cache::MemoryCache::new(NEGATIVE_CACHE_TTL_SECS));
        let bloom =
            aster_forge_cache::bloom::BloomFilter::new(aster_forge_cache::bloom::BloomConfig::new(
                INITIAL_BLOOM_CAPACITY,
                BLOOM_FALSE_POSITIVE_RATE,
            ))
            .map_err(|error| ShortlinkerError::cache_connection(error.to_string()))?;

        Ok(Arc::new(Self {
            bloom: Arc::new(bloom),
            objects,
            negatives,
            object_prefix: config.cache.redis.key_prefix.clone(),
            metrics,
            storage,
        }))
    }

    fn object_key(&self, key: &str) -> String {
        format!("{}{}", self.object_prefix, key)
    }

    fn negative_key(key: &str) -> String {
        format!("{NEGATIVE_CACHE_PREFIX}{key}")
    }
}

#[async_trait]
impl LinkCache for ForgeLinkCache {
    async fn get(&self, key: &str) -> LinkCacheLookup {
        let bloom_start = Instant::now();
        if !self.bloom.contains(key) {
            self.metrics.observe_cache_operation(
                "get",
                "bloom_filter",
                bloom_start.elapsed().as_secs_f64(),
            );
            self.metrics.inc_cache_hit("bloom_filter");
            return LinkCacheLookup::NotFound;
        }

        let negative_start = Instant::now();
        if self
            .negatives
            .get_bytes(&Self::negative_key(key))
            .await
            .is_some()
        {
            self.metrics.observe_cache_operation(
                "get",
                "negative_cache",
                negative_start.elapsed().as_secs_f64(),
            );
            self.metrics.inc_cache_hit("negative_cache");
            return LinkCacheLookup::NotFound;
        }

        let object_start = Instant::now();
        let object_key = self.object_key(key);
        let result = match self.objects.get_bytes(&object_key).await {
            Some(bytes) => match serde_json::from_slice(&bytes) {
                Ok(link) => LinkCacheLookup::Found(link),
                Err(error) => {
                    tracing::error!(key, error = %error, "invalid short link cache payload");
                    self.objects.delete(&object_key).await;
                    LinkCacheLookup::Miss
                }
            },
            None => LinkCacheLookup::Miss,
        };
        self.metrics.observe_cache_operation(
            "get",
            "object_cache",
            object_start.elapsed().as_secs_f64(),
        );
        match result {
            LinkCacheLookup::Found(_) => self.metrics.inc_cache_hit("object_cache"),
            LinkCacheLookup::Miss | LinkCacheLookup::NotFound => {
                self.metrics.inc_cache_miss("object_cache");
            }
        }
        result
    }

    async fn insert(&self, key: &str, value: ShortLink, ttl_secs: Option<u64>) {
        let start = Instant::now();
        self.negatives.delete(&Self::negative_key(key)).await;
        self.bloom.insert(key);

        match serde_json::to_vec(&value) {
            Ok(bytes) => {
                self.objects
                    .set_bytes(&self.object_key(key), bytes, ttl_secs)
                    .await;
            }
            Err(error) => {
                tracing::error!(key, error = %error, "failed to serialize short link cache payload");
            }
        }
        self.metrics.observe_cache_operation(
            "insert",
            "object_cache",
            start.elapsed().as_secs_f64(),
        );
    }

    async fn remove(&self, key: &str) {
        let start = Instant::now();
        self.objects.delete(&self.object_key(key)).await;
        self.negatives
            .set_bytes(
                &Self::negative_key(key),
                Vec::new(),
                Some(NEGATIVE_CACHE_TTL_SECS),
            )
            .await;
        self.metrics.observe_cache_operation(
            "remove",
            "object_cache",
            start.elapsed().as_secs_f64(),
        );
    }

    async fn invalidate_all(&self) {
        self.objects.invalidate_prefix(&self.object_prefix).await;
        self.negatives
            .invalidate_prefix(NEGATIVE_CACHE_PREFIX)
            .await;
    }

    async fn rebuild_all(&self) -> Result<()> {
        let count = usize::try_from(self.storage.count().await?).map_err(|_| {
            ShortlinkerError::cache_connection("link count exceeds Bloom filter capacity")
        })?;
        let mut rebuild = self
            .bloom
            .start_rebuild(aster_forge_cache::bloom::BloomConfig::new(
                count,
                BLOOM_FALSE_POSITIVE_RATE,
            ))
            .map_err(|error| ShortlinkerError::cache_connection(error.to_string()))?;
        let mut code_stream = self
            .storage
            .stream_all_codes_cursor(BLOOM_REBUILD_BATCH_SIZE);
        while let Some(batch) = code_stream.next().await {
            let batch = batch?;
            rebuild.insert_many(batch.iter().map(String::as_str));
        }
        let loaded = rebuild.commit();
        tracing::debug!(loaded, "Bloom filter rebuild completed");

        self.invalidate_all().await;
        Ok(())
    }

    async fn mark_not_found(&self, key: &str) {
        self.negatives
            .set_bytes(
                &Self::negative_key(key),
                Vec::new(),
                Some(NEGATIVE_CACHE_TTL_SECS),
            )
            .await;
    }

    async fn bloom_check(&self, key: &str) -> bool {
        self.bloom.contains(key)
    }

    async fn health_check(&self) -> LinkCacheHealth {
        let cache_type = self.objects.backend_name().to_string();
        match self.objects.health_check().await {
            Ok(()) => LinkCacheHealth {
                status: "healthy".to_string(),
                cache_type,
                bloom_filter_enabled: true,
                negative_cache_enabled: true,
                error: None,
            },
            Err(error) => LinkCacheHealth {
                status: "unhealthy".to_string(),
                cache_type,
                bloom_filter_enabled: true,
                negative_cache_enabled: true,
                error: Some(error.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Once};

    use aster_forge_cache::CacheBackend;
    use chrono::Utc;
    use tempfile::TempDir;

    use super::*;
    use crate::metrics::NoopMetrics;

    static INIT: Once = Once::new();

    fn test_link(code: &str) -> ShortLink {
        ShortLink {
            code: code.to_string(),
            target: format!("https://{code}.example.com"),
            created_at: Utc::now(),
            expires_at: None,
            password: None,
            click: 0,
        }
    }

    async fn test_cache(object_prefix: &str) -> (ForgeLinkCache, TempDir) {
        INIT.call_once(crate::config::init_config);

        let temp_dir = TempDir::new().expect("temporary cache test directory should be created");
        let db_path = temp_dir.path().join("link_cache.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
        let storage = Arc::new(
            SeaOrmStorage::new(&db_url, "sqlite", NoopMetrics::arc())
                .await
                .expect("cache test storage should be created"),
        );

        let bloom = Arc::new(
            aster_forge_cache::bloom::BloomFilter::new(aster_forge_cache::bloom::BloomConfig::new(
                100,
                BLOOM_FALSE_POSITIVE_RATE,
            ))
            .expect("cache test Bloom filter should be created"),
        );
        let objects: Arc<dyn CacheBackend> = Arc::new(aster_forge_cache::MemoryCache::new(60));
        let negatives: Arc<dyn CacheBackend> = Arc::new(aster_forge_cache::MemoryCache::new(60));

        (
            ForgeLinkCache {
                bloom,
                objects,
                negatives,
                object_prefix: object_prefix.to_string(),
                metrics: NoopMetrics::arc(),
                storage,
            },
            temp_dir,
        )
    }

    #[tokio::test]
    async fn bloom_negative_short_circuits_lookup() {
        let (cache, _temp_dir) = test_cache("links:").await;

        assert!(matches!(
            cache.get("absent").await,
            LinkCacheLookup::NotFound
        ));
    }

    #[tokio::test]
    async fn bloom_positive_without_object_is_a_miss() {
        let (cache, _temp_dir) = test_cache("links:").await;
        cache.bloom.insert("uncached");

        assert!(matches!(cache.get("uncached").await, LinkCacheLookup::Miss));
    }

    #[tokio::test]
    async fn insert_populates_bloom_and_object_cache() {
        let (cache, _temp_dir) = test_cache("links:").await;
        let link = test_link("cached");

        cache.insert("cached", link.clone(), Some(60)).await;

        assert!(cache.bloom_check("cached").await);
        match cache.get("cached").await {
            LinkCacheLookup::Found(found) => {
                assert_eq!(found.code, link.code);
                assert_eq!(found.target, link.target);
            }
            other => panic!("expected cached link, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn negative_cache_precedes_an_object_entry() {
        let (cache, _temp_dir) = test_cache("links:").await;
        cache
            .insert("negative", test_link("negative"), Some(60))
            .await;
        cache.mark_not_found("negative").await;

        assert!(matches!(
            cache.get("negative").await,
            LinkCacheLookup::NotFound
        ));
    }

    #[tokio::test]
    async fn insert_clears_a_previous_negative_entry() {
        let (cache, _temp_dir) = test_cache("links:").await;
        cache.bloom.insert("restored");
        cache.mark_not_found("restored").await;
        assert!(
            cache
                .negatives
                .get_bytes(&ForgeLinkCache::negative_key("restored"))
                .await
                .is_some()
        );

        cache
            .insert("restored", test_link("restored"), Some(60))
            .await;

        assert!(
            cache
                .negatives
                .get_bytes(&ForgeLinkCache::negative_key("restored"))
                .await
                .is_none()
        );
        assert!(matches!(
            cache.get("restored").await,
            LinkCacheLookup::Found(_)
        ));
    }

    #[tokio::test]
    async fn remove_deletes_object_and_marks_negative() {
        let (cache, _temp_dir) = test_cache("links:").await;
        cache
            .insert("removed", test_link("removed"), Some(60))
            .await;

        cache.remove("removed").await;

        assert!(cache.bloom_check("removed").await);
        assert!(
            cache
                .objects
                .get_bytes(&cache.object_key("removed"))
                .await
                .is_none()
        );
        assert!(matches!(
            cache.get("removed").await,
            LinkCacheLookup::NotFound
        ));
    }

    #[tokio::test]
    async fn corrupt_object_payload_is_deleted_and_treated_as_a_miss() {
        let (cache, _temp_dir) = test_cache("links:").await;
        cache.bloom.insert("corrupt");
        let object_key = cache.object_key("corrupt");
        cache
            .objects
            .set_bytes(&object_key, b"not-json".to_vec(), Some(60))
            .await;

        assert!(matches!(cache.get("corrupt").await, LinkCacheLookup::Miss));
        assert!(cache.objects.get_bytes(&object_key).await.is_none());
    }

    #[tokio::test]
    async fn zero_ttl_does_not_leave_an_object_entry() {
        let (cache, _temp_dir) = test_cache("links:").await;

        cache.insert("expired", test_link("expired"), Some(0)).await;

        assert!(cache.bloom_check("expired").await);
        assert!(matches!(cache.get("expired").await, LinkCacheLookup::Miss));
    }

    #[tokio::test]
    async fn invalidate_all_is_prefix_scoped_and_keeps_bloom_membership() {
        let (cache, _temp_dir) = test_cache("links:").await;
        cache.insert("owned", test_link("owned"), Some(60)).await;
        cache.bloom.insert("negative");
        cache.mark_not_found("negative").await;
        cache
            .objects
            .set_bytes("other:keep", b"object".to_vec(), Some(60))
            .await;
        cache
            .negatives
            .set_bytes("other:negative", Vec::new(), Some(60))
            .await;

        cache.invalidate_all().await;

        assert!(cache.bloom_check("owned").await);
        assert!(cache.objects.get_bytes("links:owned").await.is_none());
        assert!(
            cache
                .negatives
                .get_bytes(&ForgeLinkCache::negative_key("negative"))
                .await
                .is_none()
        );
        assert_eq!(
            cache.objects.get_bytes("other:keep").await,
            Some(b"object".to_vec())
        );
        assert!(cache.negatives.get_bytes("other:negative").await.is_some());
    }

    #[tokio::test]
    async fn rebuild_streams_database_codes_and_invalidates_product_entries() {
        let (cache, _temp_dir) = test_cache("links:").await;
        cache
            .storage
            .set(test_link("database-code"))
            .await
            .expect("database link should be stored");
        cache.insert("stale", test_link("stale"), Some(60)).await;
        cache.bloom.insert("database-code");
        cache.mark_not_found("database-code").await;
        cache
            .objects
            .set_bytes("other:keep", b"object".to_vec(), Some(60))
            .await;

        cache
            .rebuild_all()
            .await
            .expect("Bloom rebuild should succeed");

        assert!(cache.bloom_check("database-code").await);
        assert!(cache.objects.get_bytes("links:stale").await.is_none());
        assert!(
            cache
                .negatives
                .get_bytes(&ForgeLinkCache::negative_key("database-code"))
                .await
                .is_none()
        );
        assert_eq!(
            cache.objects.get_bytes("other:keep").await,
            Some(b"object".to_vec())
        );
    }

    #[tokio::test]
    async fn memory_backend_health_reports_active_capabilities() {
        let (cache, _temp_dir) = test_cache("links:").await;

        let health = cache.health_check().await;

        assert_eq!(health.status, "healthy");
        assert_eq!(health.cache_type, "memory");
        assert!(health.bloom_filter_enabled);
        assert!(health.negative_cache_enabled);
        assert!(health.error.is_none());
    }
}
