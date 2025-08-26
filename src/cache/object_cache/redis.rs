use async_trait::async_trait;
use redis::{aio::MultiplexedConnection, AsyncCommands};
use serde_json;
use tracing::{debug, error, warn};

use crate::cache::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;
use crate::storages::ShortLink;

declare_object_cache_plugin!("redis", RedisObjectCache);

pub struct RedisObjectCache {
    client: Option<redis::Client>,
    key_prefix: String,
    ttl: u64, // TTL in seconds
}

impl RedisObjectCache {
    pub fn new() -> Self {
        let config = crate::config::get_config();
        let redis_url = config.cache.redis_url.clone();
        let key_prefix = config.cache.redis_key_prefix.clone();
        let ttl = config.cache.redis_ttl;

        debug!(
            "RedisObjectCache created with prefix: '{}', TTL: {}s",
            key_prefix, ttl
        );

        let client = redis::Client::open(redis_url)
            .expect("Failed to create Redis client. Check REDIS_URL.")
            .into();

        Self {
            client,
            key_prefix,
            ttl,
        }
    }

    async fn get_connection(&self) -> Result<MultiplexedConnection, redis::RedisError> {
        let client = if let Some(ref client) = self.client {
            client
        } else {
            return Err(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Redis client not available",
            )));
        };

        let conn = client.get_multiplexed_tokio_connection().await?;
        Ok(conn)
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }

    async fn serialize_link(&self, link: &ShortLink) -> Result<String, serde_json::Error> {
        serde_json::to_string(link)
    }

    async fn deserialize_link(&self, data: &str) -> Result<ShortLink, serde_json::Error> {
        serde_json::from_str(data)
    }
}

#[async_trait]
impl ObjectCache for RedisObjectCache {
    async fn get(&self, key: &str) -> CacheResult {
        let redis_key = self.make_key(key);

        let mut conn = match self.get_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return CacheResult::ExistsButNoValue;
            }
        };

        let result: redis::RedisResult<Option<String>> = conn.get(redis_key).await;

        match result {
            Ok(Some(data)) => match self.deserialize_link(&data).await {
                Ok(link) => {
                    debug!("Successfully retrieved key: {}", key);
                    CacheResult::Found(link)
                }
                Err(e) => {
                    error!("Failed to deserialize ShortLink for key '{}': {}", key, e);
                    CacheResult::ExistsButNoValue
                }
            },
            Ok(None) => {
                debug!("Key not found in cache: {}", key);
                CacheResult::NotFound
            }
            Err(e) => {
                error!("Failed to get key '{}': {}", key, e);
                CacheResult::ExistsButNoValue
            }
        }
    }

    async fn insert(&self, key: String, value: ShortLink) {
        let redis_key = self.make_key(&key);

        let mut conn = match self.get_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return;
            }
        };

        match self.serialize_link(&value).await {
            Ok(serialized_value) => {
                match conn
                    .set_ex::<String, String, ()>(redis_key, serialized_value, self.ttl)
                    .await
                {
                    Ok(_) => {
                        debug!("Successfully inserted key into cache: {}", key);
                    }
                    Err(e) => {
                        error!("Failed to insert key '{}' into cache: {}", key, e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to serialize ShortLink for key '{}': {}", key, e);
            }
        }
    }

    async fn remove(&self, key: &str) {
        let redis_key = self.make_key(key);

        let mut conn = match self.get_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                return;
            }
        };

        match conn.del::<String, i32>(redis_key).await {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    debug!("Successfully removed key from cache: {}", key);
                } else {
                    debug!("Key not found in cache for removal: {}", key);
                }
            }
            Err(e) => {
                error!("Failed to remove key '{}': {}", key, e);
            }
        }
    }

    async fn invalidate_all(&self) {
        warn!("RedisObjectCache does not implement invalidate_all");
        return;
    }
}
