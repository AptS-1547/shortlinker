use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
pub use crate::structs::{ShortLink, StorageSerializableShortLink};

use crate::errors::Result;
use async_trait::async_trait;


#[async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, code: &str) -> Option<ShortLink>;
    async fn load_all(&self) -> HashMap<String, ShortLink>;
    async fn set(&self, link: ShortLink) -> Result<()>;
    async fn remove(&self, code: &str) -> Result<()>;
    async fn reload(&self) -> Result<()>;
    async fn get_backend_name(&self) -> String;

    /// 增加点击量计数器
    async fn increment_click(&self, code: &str) -> Result<()>;
}

pub mod file;
pub mod sled;
pub mod sqlite;

pub struct StorageFactory;

impl StorageFactory {
    pub async fn create() -> Result<Arc<dyn Storage>> {
        let backend = env::var("STORAGE_BACKEND").unwrap_or_else(|_| "sqlite".into());

        let boxed: Box<dyn Storage> = match backend.as_str() {
            "sled" => Box::new(sled::SledStorage::new_async().await?),
            "file" => Box::new(file::FileStorage::new_async().await?),
            _ => Box::new(sqlite::SqliteStorage::new_async().await?),
        };

        Ok(Arc::from(boxed))
    }
}
