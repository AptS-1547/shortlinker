use async_trait::async_trait;
use tracing::trace;

use crate::cache::ExistenceFilter;
use crate::declare_existence_filter_plugin;

declare_existence_filter_plugin!("null", NullExistenceFilterPlugin);

pub struct NullExistenceFilterPlugin;

impl Default for NullExistenceFilterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl NullExistenceFilterPlugin {
    pub fn new() -> Self {
        trace!("Using NullExistenceFilterPlugin: no L1 cache will be used");
        NullExistenceFilterPlugin
    }
}

#[async_trait]
impl ExistenceFilter for NullExistenceFilterPlugin {
    async fn check(&self, _key: &str) -> bool {
        trace!("NullExistenceFilterPlugin: always return true for check");
        true
    }

    async fn set(&self, _key: &str) {
        trace!("NullExistenceFilterPlugin: skip set");
    }

    async fn bulk_set(&self, _keys: &[String]) {
        trace!("NullExistenceFilterPlugin: skip bulk_set");
    }

    async fn clear(&self, _count: usize, _fp_rate: f64) {
        trace!("NullExistenceFilterPlugin: skip clear");
    }
}
