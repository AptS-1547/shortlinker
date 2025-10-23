#[macro_use]
mod macros;

use std::collections::HashMap;
use std::sync::Arc;
use tracing::error;

use crate::{
    errors::{Result, ShortlinkerError},
    repository::models::StorageConfig,
};

pub mod backends;
pub mod click;
pub mod models;
pub mod register;

pub use models::ShortLink;
use register::get_repository_plugin;

#[async_trait::async_trait]
pub trait Repository: Send + Sync {
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
}

pub struct RepositoryFactory;

impl RepositoryFactory {
    pub async fn create() -> Result<Arc<dyn Repository>> {
        let config = crate::system::app_config::get_config();
        let backend = &config.database.backend;

        if let Some(ctor) = get_repository_plugin(backend) {
            let repository = ctor().await?;
            Ok(Arc::from(repository))
        } else {
            error!("Failed to create repository backend: {}", backend);
            let available_backends = register::get_repository_plugin_names();
            error!("Available repository backends: {:?}", available_backends);
            Err(ShortlinkerError::storage_plugin_not_found(format!(
                "Unknown repository backend: {}",
                backend
            )))
        }
    }
}
