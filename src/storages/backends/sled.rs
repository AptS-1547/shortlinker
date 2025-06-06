use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use super::{ShortLink, Storage};
use crate::errors::Result;

// 注册 sled 存储插件
// 这样子可以在应用启动时自动注册 sled 存储插件
pub fn register_sled_plugin() {
    use super::register::register_storage_plugin;
    register_storage_plugin(
        "sled",
        Arc::new(|| {
            Box::pin(async {
                let s = SledStorage::new_async().await?;
                Ok(Box::new(s) as Box<dyn Storage>)
            })
        }),
    );
}

pub struct SledStorage;

impl SledStorage {
    pub async fn new_async() -> Result<Self> {
        Ok(SledStorage)
    }
}

#[async_trait]
impl Storage for SledStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        println!("FileStorage::get called with {}", code);
        let link = ShortLink {
            code: "example_code".to_string(),
            target: "http://example.com".to_string(),
            expires_at: None,
            created_at: chrono::Utc::now(),
        };

        Some(link)
    }

    async fn load_all(&self) -> HashMap<String, ShortLink> {
        println!("SledStorage::get_all called");
        let mut links = HashMap::new();
        links.insert(
            "example_code".to_string(),
            ShortLink {
                code: "example_code".to_string(),
                target: "http://example.com".to_string(),
                expires_at: None,
                created_at: chrono::Utc::now(),
            },
        );

        links
    }

    async fn set(&self, link: ShortLink) -> Result<()> {
        println!("SledStorage::save called with {:?}", link);
        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<()> {
        println!("SledStorage::remove called with {}", code);
        Ok(())
    }

    async fn reload(&self) -> Result<()> {
        Ok(())
    }

    async fn get_backend_name(&self) -> String {
        "sled".to_string()
    }

    async fn increment_click(&self, _code: &str) -> Result<()> {
        Ok(())
    }
}
