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

pub use models::ShortLink;

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
        let database_url = &config.database.database_url;

        match backend.as_str() {
            "sqlite" | "mysql" | "postgres" | "mariadb" => {
                let repository = backends::sea_orm::SeaOrmRepository::new(database_url, backend).await?;
                Ok(Arc::new(repository) as Arc<dyn Repository>)
            }
            "sled" => {
                let repository = backends::sled::SledStorage::new_async().await?;
                Ok(Arc::new(repository) as Arc<dyn Repository>)
            }
            _ => {
                error!("Unknown repository backend: {}", backend);
                Err(ShortlinkerError::storage_plugin_not_found(format!(
                    "Unknown repository backend: {}. Supported: sqlite, mysql, postgres, mariadb, sled",
                    backend
                )))
            }
        }
    }
}
