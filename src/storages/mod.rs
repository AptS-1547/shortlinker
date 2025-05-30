use std::collections::HashMap;
use std::env;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::errors::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct ShortLink {
    pub code: String,
    pub target: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct SerializableShortLink {
    short_code: String,
    target_url: String,
    created_at: String,
    expires_at: Option<String>,
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, code: &str) -> Option<ShortLink>;
    async fn load_all(&self) -> HashMap<String, ShortLink>;
    async fn set(&self, link: ShortLink) -> Result<()>;
    async fn remove(&self, code: &str) -> Result<()>;
    async fn reload(&self) -> Result<()>;
    async fn get_backend_name(&self) -> String;
}

pub mod file;
pub mod sled;
pub mod sqlite;

pub struct StorageFactory;

impl StorageFactory {
    pub fn create() -> Result<Arc<dyn Storage>> {
        let backend = env::var("STORAGE_BACKEND").unwrap_or_else(|_| "sqlite".into());

        let boxed: Box<dyn Storage> = match backend.as_str() {
            "sled" => Box::new(sled::SledStorage::new()?),
            "file" => Box::new(file::FileStorage::new()?),
            _ => Box::new(sqlite::SqliteStorage::new()?),
        };

        Ok(Arc::from(boxed))
    }
}
