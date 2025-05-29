use std::collections::HashMap;

use super::{ShortLink, Storage};
use async_trait::async_trait;

pub struct RedisStorage;

impl RedisStorage {
    pub fn new() -> Self {
        RedisStorage
    }
}

#[async_trait]
impl Storage for RedisStorage {
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
        println!("RedisStorage::get_all called");
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

    async fn set(&self, link: ShortLink) -> Result<(), String> {
        println!("RedisStorage::save called with {:?}", link);
        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<(), String> {
        println!("RedisStorage::remove called with {}", code);
        Ok(())
    }

    async fn reload(&self) -> Result<(), String> {
        Ok(())
    }
}
