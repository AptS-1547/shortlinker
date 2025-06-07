use async_trait::async_trait;
use tracing::debug;

use crate::cache::ExistenceFilter;
use crate::declare_existence_filter_plugin;

declare_existence_filter_plugin!("null", NullExistenceFilterL1Cache);

pub struct NullExistenceFilterL1Cache;

impl Default for NullExistenceFilterL1Cache {
    fn default() -> Self {
        Self::new()
    }
}

impl NullExistenceFilterL1Cache {
    pub fn new() -> Self {
        debug!("Using NullExistenceFilterL1Cache: no L1 cache will be used");
        NullExistenceFilterL1Cache
    }
}

#[async_trait]
impl ExistenceFilter for NullExistenceFilterL1Cache {
    async fn check(&self, _key: &str) -> bool {
        true
    }

    async fn set(&self, _key: &str) {
        // noop
    }

    async fn bulk_set(&self, _keys: &[String]) {
        debug!("NullExistenceFilterL1Cache: skip bulk_set");
    }

    async fn clear(&self, _count: usize, _fp_rate: f64) {
        debug!("NullExistenceFilterL1Cache: skip clear");
    }
}
