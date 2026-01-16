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
