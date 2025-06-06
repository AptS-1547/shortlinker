use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use crate::errors::{Result, ShortlinkerError};
use async_trait::async_trait;

mod backends;
mod models;
mod register;

pub use backends::{file, sled, sqlite};
pub use models::{SerializableShortLink, ShortLink};
use register::get_storage_plugin;

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

pub struct StorageFactory;

// 注册所有存储插件
pub fn register_all_plugins() {
    sqlite::register_sqlite_plugin();
    sled::register_sled_plugin();
    file::register_file_plugin();
}

impl StorageFactory {
    pub async fn create() -> Result<Arc<dyn Storage>> {
        // 注册所有存储插件
        register_all_plugins();

        let backend = env::var("STORAGE_BACKEND").unwrap_or_else(|_| "sqlite".into());

        if let Some(ctor) = get_storage_plugin(&backend) {
            let storage = ctor().await?;
            register::debug_registry(); // 调试函数，打印已注册的存储后端
            Ok(Arc::from(storage))
        } else {
            Err(ShortlinkerError::storage_plugin_not_found(format!(
                "Unknown storage backend: {}",
                backend
            )))
        }
    }
}
