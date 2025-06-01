use std::collections::HashMap;

use super::{ShortLink, Storage};
use crate::errors::Result;
use async_trait::async_trait;

pub struct SledStorage;

impl SledStorage {
    pub fn new() -> Result<Self> {
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

    async fn increment_click(&self, code: &str) -> Result<()> {
        Ok(())
    }
}
