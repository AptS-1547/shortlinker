use std::collections::HashMap;
use std::sync::Arc;

use crate::{errors::Result, repository::models::StorageConfig};

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
        let database_url = &config.database.database_url;

        // 从 URL 自动推断数据库类型
        let backend = backends::infer_backend_from_url(database_url)?;

        let repository = backends::sea_orm::SeaOrmRepository::new(database_url, &backend).await?;
        Ok(Arc::new(repository) as Arc<dyn Repository>)
    }
}
