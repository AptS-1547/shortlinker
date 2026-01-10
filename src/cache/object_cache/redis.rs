use async_trait::async_trait;
use redis::{AsyncCommands, aio::MultiplexedConnection};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, trace, warn};

use crate::cache::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;
use crate::storage::ShortLink;

declare_object_cache_plugin!("redis", RedisObjectCache);

pub struct RedisObjectCache {
    client: redis::Client,
    /// 持久化连接，使用 RwLock 保护
    connection: Arc<RwLock<Option<MultiplexedConnection>>>,
    key_prefix: String,
    ttl: u64,
}

impl Default for RedisObjectCache {
    fn default() -> Self {
        Self::new().expect("RedisObjectCache initialization failed")
    }
}

impl RedisObjectCache {
    pub fn new() -> Result<Self, String> {
        let config = crate::config::get_config();
        let redis_config = &config.cache.redis;

        let ttl = config.cache.default_ttl;

        debug!(
            "RedisObjectCache created with prefix: '{}', TTL: {}s",
            redis_config.key_prefix, ttl
        );

        let client = redis::Client::open(redis_config.url.clone())
            .expect("Failed to create Redis client. Check REDIS_URL.");

        // 测试 Redis 连接 - 使用同步连接进行简单测试
        match client.get_connection() {
            Ok(mut conn) => match redis::cmd("PING").query::<String>(&mut conn) {
                Ok(response) => {
                    debug!("Redis connection test successful: {}", response);
                }
                Err(e) => {
                    error!(
                        "Failed to ping Redis server: {}. Check Redis server status and URL: {}",
                        e, redis_config.url
                    );
                    return Err(format!("Redis ping failed: {e}"));
                }
            },
            Err(e) => {
                error!(
                    "Failed to ping Redis server: {}. Check Redis server status and URL: {}",
                    e, redis_config.url
                );
                return Err(format!("Redis ping failed: {e}"));
            }
        }

        Ok(Self {
            client,
            connection: Arc::new(RwLock::new(None)),
            key_prefix: redis_config.key_prefix.clone(),
            ttl: config.cache.default_ttl,
        })
    }

    /// 获取或建立持久连接
    async fn get_connection(&self) -> Result<MultiplexedConnection, redis::RedisError> {
        // 首先尝试读取现有连接
        {
            let conn_guard = self.connection.read().await;
            if let Some(ref conn) = *conn_guard {
                return Ok(conn.clone());
            }
        }

        // 需要建立新连接
        let mut conn_guard = self.connection.write().await;

        // 双重检查，避免竞态条件
        if let Some(ref conn) = *conn_guard {
            return Ok(conn.clone());
        }

        let new_conn = self.client.get_multiplexed_async_connection().await?;
        *conn_guard = Some(new_conn.clone());
        debug!("Redis connection established and cached");

        Ok(new_conn)
    }

    /// 重置连接（在连接错误时调用）
    async fn reset_connection(&self) {
        let mut conn_guard = self.connection.write().await;
        *conn_guard = None;
        debug!("Redis connection reset due to error");
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}{}", self.key_prefix, key)
    }

    fn serialize_link(link: &ShortLink) -> Result<String, serde_json::Error> {
        serde_json::to_string(link)
    }

    fn deserialize_link(data: &str) -> Result<ShortLink, serde_json::Error> {
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
                self.reset_connection().await;
                return CacheResult::ExistsButNoValue;
            }
        };

        let result: redis::RedisResult<Option<String>> = conn.get(&redis_key).await;

        match result {
            Ok(Some(data)) => match Self::deserialize_link(&data) {
                Ok(link) => {
                    trace!("Successfully retrieved key: {}", key);
                    CacheResult::Found(link)
                }
                Err(e) => {
                    error!("Failed to deserialize ShortLink for key '{}': {}", key, e);
                    CacheResult::ExistsButNoValue
                }
            },
            Ok(None) => {
                trace!("Key not found in cache: {}", key);
                CacheResult::NotFound
            }
            Err(e) => {
                error!("Failed to get key '{}': {}", key, e);
                // 连接可能已断开，重置连接
                self.reset_connection().await;
                CacheResult::ExistsButNoValue
            }
        }
    }

    async fn insert(&self, key: &str, value: ShortLink) {
        let redis_key = self.make_key(key);

        let mut conn = match self.get_connection().await {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to get Redis connection: {}", e);
                self.reset_connection().await;
                return;
            }
        };

        match Self::serialize_link(&value) {
            Ok(serialized_value) => {
                match conn
                    .set_ex::<String, String, ()>(redis_key, serialized_value, self.ttl)
                    .await
                {
                    Ok(_) => {
                        trace!("Successfully inserted key into cache: {}", key);
                    }
                    Err(e) => {
                        error!("Failed to insert key '{}' into cache: {}", key, e);
                        self.reset_connection().await;
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
                self.reset_connection().await;
                return;
            }
        };

        match conn.del::<String, i32>(redis_key).await {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    trace!("Successfully removed key from cache: {}", key);
                } else {
                    trace!("Key not found in cache for removal: {}", key);
                }
            }
            Err(e) => {
                error!("Failed to remove key '{}': {}", key, e);
                self.reset_connection().await;
            }
        }
    }

    async fn invalidate_all(&self) {
        warn!("RedisObjectCache does not implement invalidate_all");
    }
}
