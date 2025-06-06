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

impl BloomFilterL1Cache {
    pub fn new(capacity: usize, fp_rate: f64) -> Self {
        let bloom = Bloom::new_for_fp_rate(capacity, fp_rate)
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

    async fn clear(&self, count: usize) {
        let mut bloom = self.inner.write().await;
        let new = Bloom::new_for_fp_rate(count, 0.001)
            .unwrap_or_else(|_| panic!("Failed to create new bloom filter"));
        *bloom = new;
        debug!("Bloom filter cleared and reinitialized");
    }
}
