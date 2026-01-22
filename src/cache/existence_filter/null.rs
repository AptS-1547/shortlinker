use async_trait::async_trait;
use tracing::trace;

use crate::cache::ExistenceFilter;
use crate::declare_existence_filter_plugin;
use crate::errors::Result;

declare_existence_filter_plugin!("null", NullExistenceFilterPlugin);

pub struct NullExistenceFilterPlugin;

impl Default for NullExistenceFilterPlugin {
    fn default() -> Self {
        Self::new().expect("Failed to create default NullExistenceFilterPlugin")
    }
}

impl NullExistenceFilterPlugin {
    pub fn new() -> Result<Self> {
        trace!("Using NullExistenceFilterPlugin: no L1 cache will be used");
        Ok(NullExistenceFilterPlugin)
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

    async fn clear(&self, _count: usize, _fp_rate: f64) -> Result<()> {
        trace!("NullExistenceFilterPlugin: skip clear");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_succeeds() {
        let plugin = NullExistenceFilterPlugin::new();
        assert!(plugin.is_ok());
    }

    #[test]
    fn test_default_succeeds() {
        let _plugin = NullExistenceFilterPlugin;
    }

    #[tokio::test]
    async fn test_check_always_returns_true() {
        let plugin = NullExistenceFilterPlugin::new().unwrap();
        assert!(plugin.check("any_key").await);
        assert!(plugin.check("").await);
        assert!(plugin.check("nonexistent").await);
    }

    #[tokio::test]
    async fn test_set_is_noop() {
        let plugin = NullExistenceFilterPlugin::new().unwrap();
        // set 应该是空操作，不会改变 check 的结果
        plugin.set("key1").await;
        assert!(plugin.check("key1").await);
    }

    #[tokio::test]
    async fn test_bulk_set_is_noop() {
        let plugin = NullExistenceFilterPlugin::new().unwrap();
        let keys = vec!["k1".to_string(), "k2".to_string(), "k3".to_string()];
        plugin.bulk_set(&keys).await;
        // 仍然返回 true
        assert!(plugin.check("k1").await);
    }

    #[tokio::test]
    async fn test_clear_succeeds() {
        let plugin = NullExistenceFilterPlugin::new().unwrap();
        let result = plugin.clear(1000, 0.01).await;
        assert!(result.is_ok());
    }
}
