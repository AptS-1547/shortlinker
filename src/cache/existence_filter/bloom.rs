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
        let capacity = count.max(9_000) + 1000;
        *bloom = Bloom::new_for_fp_rate(capacity, fp_rate).map_err(|e| {
            ShortlinkerError::cache_connection(format!("Failed to clear bloom filter: {e}"))
        })?;
        debug!(
            "Bloom filter cleared with count: {}, fp_rate: {}",
            capacity, fp_rate
        );
        Ok(())
    }
}
