use async_trait::async_trait;
use bloomfilter::Bloom;
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::debug;

use crate::cache::ExistenceFilter;
use crate::declare_existence_filter_plugin;
use crate::errors::{Result, ShortlinkerError};

declare_existence_filter_plugin!("bloom", BloomExistenceFilterPlugin);

pub struct BloomExistenceFilterPlugin {
    inner: Arc<RwLock<Bloom<str>>>,
}

impl Default for BloomExistenceFilterPlugin {
    fn default() -> Self {
        Self::new().expect("Failed to create default BloomExistenceFilterPlugin")
    }
}

impl BloomExistenceFilterPlugin {
    pub fn new() -> Result<Self> {
        let bloom = Bloom::new_for_fp_rate(10_000, 0.001).map_err(|e| {
            ShortlinkerError::cache_connection(format!("Failed to create bloom filter: {e}"))
        })?;
        Ok(Self {
            inner: Arc::new(RwLock::new(bloom)),
        })
    }
}

#[async_trait]
impl ExistenceFilter for BloomExistenceFilterPlugin {
    async fn check(&self, key: &str) -> bool {
        let bloom = self.inner.read();
        bloom.check(key)
    }

    async fn set(&self, key: &str) {
        let mut bloom = self.inner.write();
        bloom.set(key);
    }

    async fn bulk_set(&self, keys: &[String]) {
        let mut bloom = self.inner.write();
        for key in keys {
            bloom.set(key);
        }
        debug!("Bulk inserted {} keys into bloom filter", keys.len());
    }

    async fn clear(&self, count: usize, fp_rate: f64) -> Result<()> {
        let mut bloom = self.inner.write();
        // 预留 20% 空间，但至少预留 1000
        let reserve = (count / 5).max(1000);
        let capacity = count + reserve;
        *bloom = Bloom::new_for_fp_rate(capacity, fp_rate).map_err(|e| {
            ShortlinkerError::cache_connection(format!("Failed to clear bloom filter: {e}"))
        })?;
        debug!(
            "Bloom filter cleared with capacity: {} (count: {} + reserve: {}), fp_rate: {}",
            capacity, count, reserve, fp_rate
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_empty_returns_false() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();
        assert!(!filter.check("nonexistent").await);
    }

    #[tokio::test]
    async fn test_set_and_check() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();

        filter.set("test_key").await;
        assert!(filter.check("test_key").await);

        // 未设置的 key 应该返回 false（大概率）
        assert!(!filter.check("other_key").await);
    }

    #[tokio::test]
    async fn test_bulk_set() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();
        let keys: Vec<String> = (0..100).map(|i| format!("key_{}", i)).collect();

        filter.bulk_set(&keys).await;

        // 所有设置的 key 都应该存在
        for key in &keys {
            assert!(filter.check(key).await, "Key {} should exist", key);
        }
    }

    #[tokio::test]
    async fn test_clear_resets_filter() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();

        filter.set("test_key").await;
        assert!(filter.check("test_key").await);

        filter.clear(1000, 0.001).await.unwrap();

        // 清除后，之前的 key 应该不存在了
        assert!(!filter.check("test_key").await);
    }

    #[tokio::test]
    async fn test_unicode_key() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();

        filter.set("中文键").await;
        assert!(filter.check("中文键").await);

        filter.set("🎉emoji").await;
        assert!(filter.check("🎉emoji").await);
    }

    #[tokio::test]
    async fn test_false_positive_rate_within_bounds() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();

        // 插入 1000 个 key
        let keys: Vec<String> = (0..1000).map(|i| format!("existing_{}", i)).collect();
        filter.bulk_set(&keys).await;

        // 测试 10000 个不存在的 key，统计误报率
        let mut false_positives = 0;
        for i in 0..10000 {
            if filter.check(&format!("nonexistent_{}", i)).await {
                false_positives += 1;
            }
        }

        // 默认 fp_rate 是 0.001，允许一定误差
        // 10000 次查询，期望误报约 10 次，允许最多 50 次（0.5%）
        assert!(
            false_positives < 50,
            "False positive rate too high: {}/10000",
            false_positives
        );
    }

    #[tokio::test]
    async fn test_concurrent_set_and_check() {
        use std::sync::Arc;

        let filter = Arc::new(BloomExistenceFilterPlugin::new().unwrap());
        let mut handles = vec![];

        // 并发设置 100 个 key
        for i in 0..100 {
            let f = Arc::clone(&filter);
            handles.push(tokio::spawn(async move {
                f.set(&format!("concurrent_key_{}", i)).await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // 验证所有 key 都存在
        for i in 0..100 {
            assert!(
                filter.check(&format!("concurrent_key_{}", i)).await,
                "Key {} should exist after concurrent set",
                i
            );
        }
    }
}
