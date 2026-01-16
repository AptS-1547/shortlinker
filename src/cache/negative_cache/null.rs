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
