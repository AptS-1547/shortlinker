use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::env;

use async_trait::async_trait;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct ShortLink {
    pub code: String,
    pub target: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, code: &str) -> Option<ShortLink>;
    async fn load_all(&self) -> HashMap<String, ShortLink>;
    async fn set(&self, link: ShortLink) -> Result<(), String>;
    async fn remove(&self, code: &str) -> Result<(), String>;
}

pub mod file;
pub mod redis;

pub static STORAGE: Lazy<Arc<dyn Storage>> = Lazy::new(|| {
    let backend = env::var("STORAGE_BACKEND").unwrap_or_else(|_| "file".into());

    let boxed: Box<dyn Storage> = match backend.as_str() {
        "redis" => Box::new(redis::RedisStorage::new()),
        _ => Box::new(file::FileStorage::new()),
    };

    Arc::from(boxed)
});
