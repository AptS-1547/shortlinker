use std::sync::Arc;

use crate::errors::Result;

pub mod backend;
pub mod models;

pub use backend::SeaOrmStorage;
pub use models::ShortLink;

pub struct StorageFactory;

impl StorageFactory {
    pub async fn create() -> Result<Arc<SeaOrmStorage>> {
        let config = crate::config::get_config();
        let database_url = &config.database.database_url;

        // 从 URL 自动推断数据库类型
        let backend_type = backend::infer_backend_from_url(database_url)?;

        let storage = backend::SeaOrmStorage::new(database_url, &backend_type).await?;
        Ok(Arc::new(storage))
    }
}
