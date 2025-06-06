use async_trait::async_trait;
use bloomfilter::Bloom;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::cache::L1Cache;
use crate::declare_l1_plugin;

declare_l1_plugin!("bloom", BloomFilterL1Cache);

pub struct BloomFilterL1Cache {
    inner: Arc<RwLock<Bloom<str>>>,
}

impl Default for BloomFilterL1Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl BloomFilterL1Cache {
    pub fn new() -> Self {
        let bloom = Bloom::new_for_fp_rate(10_000, 0.001)
            .unwrap_or_else(|_| panic!("Failed to create bloom filter"));
        Self {
            inner: Arc::new(RwLock::new(bloom)),
        }
    }
}

#[async_trait]
impl L1Cache for BloomFilterL1Cache {
    async fn check(&self, key: &str) -> bool {
        let bloom = self.inner.read().await;
        bloom.check(key)
    }

    async fn set(&self, key: &str) {
        let mut bloom = self.inner.write().await;
        bloom.set(key);
    }

    async fn bulk_set(&self, keys: &[String]) {
        let mut bloom = self.inner.write().await;
        for key in keys {
            bloom.set(key);
        }
        debug!("Bulk inserted {} keys into bloom filter", keys.len());
    }

    async fn clear(&self, count: usize, _fp_rate: f64) {
        let mut bloom = self.inner.write().await;
        let capacity = count.max(9_000) + 1000; // Ensure a minimum capacity
        *bloom = Bloom::new_for_fp_rate(capacity, _fp_rate)
            .unwrap_or_else(|_| panic!("Failed to clear bloom filter"));
        debug!(
            "Bloom filter cleared with count: {}, fp_rate: {}",
            capacity, _fp_rate
        );
    }
}
