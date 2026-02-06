use std::sync::Arc;

use crate::errors::Result;

pub mod backend;
pub mod config_store;
pub mod models;

pub use backend::{LinkFilter, SeaOrmStorage};
pub use config_store::{ConfigHistoryEntry, ConfigItem, ConfigStore, ConfigUpdateResult};
pub use models::{LinkStats, ShortLink};

#[cfg(feature = "metrics")]
use crate::metrics::MetricsRecorder;

pub struct StorageFactory;

impl StorageFactory {
    #[cfg(feature = "metrics")]
    pub async fn create(metrics: Arc<dyn MetricsRecorder>) -> Result<Arc<SeaOrmStorage>> {
        let config = crate::config::get_config();
        let database_url = &config.database.database_url;

        // 从 URL 自动推断数据库类型
        let backend_type = backend::infer_backend_from_url(database_url)?;

        let storage = backend::SeaOrmStorage::new(database_url, &backend_type, metrics).await?;
        Ok(Arc::new(storage))
    }

    #[cfg(not(feature = "metrics"))]
    pub async fn create() -> Result<Arc<SeaOrmStorage>> {
        let config = crate::config::get_config();
        let database_url = &config.database.database_url;

        // 从 URL 自动推断数据库类型
        let backend_type = backend::infer_backend_from_url(database_url)?;

        let storage = backend::SeaOrmStorage::new(database_url, &backend_type).await?;
        Ok(Arc::new(storage))
    }
}
