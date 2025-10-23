use async_trait::async_trait;
use std::collections::HashMap;

use crate::repository::{ShortLink, Repository};
use crate::{errors::Result, repository::models::StorageConfig};

// 注册 sled 存储插件
// 这样子可以在应用启动时自动注册 sled 存储插件
declare_repository_plugin!("sled", SledStorage);

pub struct SledStorage;

impl SledStorage {
    pub async fn new_async() -> Result<Self> {
        Ok(SledStorage)
    }
}

#[async_trait]
impl Repository for SledStorage {
    async fn get(&self, code: &str) -> Option<ShortLink> {
        println!("FileStorage::get called with {}", code);
        let link = ShortLink {
            code: "example_code".to_string(),
            target: "http://example.com".to_string(),
            expires_at: None,
            created_at: chrono::Utc::now(),
            password: None,
            click: 0, // 默认点击量为0
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
                password: None,
                click: 0, // 默认点击量为0
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

    async fn get_backend_config(&self) -> StorageConfig {
        StorageConfig {
            storage_type: "sled".into(),
            support_click: true,
        }
    }
}
