use async_trait::async_trait;
use tracing::debug;

use crate::cache::L1Cache;
use crate::declare_l1_plugin;

declare_l1_plugin!("null", NullFilterL1Cache);

pub struct NullFilterL1Cache;

impl NullFilterL1Cache {
    pub fn new(_capacity: usize, _fp_rate: f64) -> Self {
        debug!("Using NullFilterL1Cache: no L1 cache will be used");
        NullFilterL1Cache
    }
}

#[async_trait]
impl L1Cache for NullFilterL1Cache {
    async fn check(&self, _key: &str) -> bool {
        true
    }

    async fn set(&self, _key: &str) {
        // noop
    }

    async fn bulk_set(&self, _keys: &[String]) {
        debug!("NullFilterL1Cache: skip bulk_set");
    }

    async fn clear(&self, _count: usize) {
        debug!("NullFilterL1Cache: skip clear");
    }
}
