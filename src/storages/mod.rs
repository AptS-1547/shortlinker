#[macro_use]
mod macros;

use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tracing::error;

use crate::{
    errors::{Result, ShortlinkerError},
    storages::models::StorageConfig,
};

pub mod backends;
pub mod click;
pub mod models;
pub mod register;

pub use models::{CachePreference, ShortLink};
use register::get_storage_plugin;

#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, code: &str) -> Option<ShortLink>;
    async fn load_all(&self) -> HashMap<String, ShortLink>;
    async fn set(&self, link: ShortLink) -> Result<()>;
    async fn remove(&self, code: &str) -> Result<()>;
    async fn reload(&self) -> Result<()>;
    async fn get_backend_config(&self) -> StorageConfig;

    /// 增加点击量计数器
    fn as_click_sink(&self) -> Option<Arc<dyn click::ClickSink>> {
        None
    }

    /// 缓存首选项
    fn preferred_cache(&self) -> CachePreference {
        CachePreference {
            l1: "null".into(),
            l2: "null".into(),
        }
    }
}

pub struct StorageFactory;

impl StorageFactory {
    pub async fn create() -> Result<Arc<dyn Storage>> {
        let backend = env::var("STORAGE_BACKEND").unwrap_or_else(|_| "sqlite".into());

        if let Some(ctor) = get_storage_plugin(&backend) {
            let storage = ctor().await?;
            Ok(Arc::from(storage))
        } else {
            error!("Failed to create storage backend: {}", backend);
            let available_backends = register::get_storage_plugin_names();
            error!("Available storage backends: {:?}", available_backends);
            Err(ShortlinkerError::storage_plugin_not_found(format!(
                "Unknown storage backend: {}",
                backend
            )))
        }
    }
}
