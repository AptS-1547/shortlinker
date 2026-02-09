use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use tracing::{debug, error, trace};

use crate::cache::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;
use crate::storage::ShortLink;

declare_object_cache_plugin!("redis", RedisObjectCache);

pub struct RedisObjectCache {
    /// ConnectionManager 自动处理重连
    connection: ConnectionManager,
    key_prefix: String,
    ttl: u64,
}

impl RedisObjectCache {
    pub async fn new() -> Result<Self, String> {
        let config = crate::config::get_config();
        let redis_config = &config.cache.redis;
        let ttl = config.cache.default_ttl;

        debug!(
            "Initializing RedisObjectCache with prefix: '{}', TTL: {}s",
            redis_config.key_prefix, ttl
        );

        let client = redis::Client::open(redis_config.url.clone())
            .map_err(|e| format!("Failed to create Redis client: {e}"))?;

        // 使用 ConnectionManager，支持自动重连
        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| format!("Failed to create Redis ConnectionManager: {e}"))?;

        debug!(
            "RedisObjectCache initialized with ConnectionManager, prefix: '{}', TTL: {}s",
            redis_config.key_prefix, ttl
        );

        Ok(Self {
            connection,
            key_prefix: redis_config.key_prefix.clone(),
            ttl,
        })
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

        // ConnectionManager 可以直接 clone 使用，自动处理重连
        let mut conn = self.connection.clone();

        let result: redis::RedisResult<Option<String>> = conn.get(&redis_key).await;

        match result {
            Ok(Some(data)) => match Self::deserialize_link(&data) {
                Ok(link) => {
                    trace!("Successfully retrieved key: {}", key);
                    CacheResult::Found(link)
                }
                Err(e) => {
                    error!("Failed to deserialize ShortLink for key '{}': {}", key, e);
                    // 删除损坏的数据
                    let _ = conn.del::<&str, ()>(&redis_key).await;
                    CacheResult::Miss
                }
            },
            Ok(None) => {
                trace!("Key not found in cache: {}", key);
                CacheResult::Miss
            }
            Err(e) => {
                // ConnectionManager 自动处理重连，这里只记录错误
                error!("Redis get error (will auto-reconnect): {}", e);
                CacheResult::Miss
            }
        }
    }

    async fn insert(&self, key: &str, value: ShortLink, ttl_secs: Option<u64>) {
        let redis_key = self.make_key(key);
        let mut conn = self.connection.clone();
        let ttl = ttl_secs.unwrap_or(self.ttl);

        match Self::serialize_link(&value) {
            Ok(serialized_value) => {
                match conn
                    .set_ex::<String, String, ()>(redis_key, serialized_value, ttl)
                    .await
                {
                    Ok(_) => {
                        trace!(
                            "Successfully inserted key into cache: {} (TTL: {}s)",
                            key, ttl
                        );
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
        let mut conn = self.connection.clone();

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
            }
        }
    }

    async fn invalidate_all(&self) {
        let mut conn = self.connection.clone();
        let pattern = format!("{}*", self.key_prefix);

        // 使用 Lua 脚本原子性删除所有匹配的 key
        // 这避免了 SCAN+DEL 模式中新 key 可能被漏删的问题
        let lua_script = r#"
            local cursor = "0"
            local deleted = 0
            repeat
                local result = redis.call("SCAN", cursor, "MATCH", ARGV[1], "COUNT", 100)
                cursor = result[1]
                local keys = result[2]
                if #keys > 0 then
                    deleted = deleted + redis.call("DEL", unpack(keys))
                end
            until cursor == "0"
            return deleted
        "#;

        let result: redis::RedisResult<u64> = redis::Script::new(lua_script)
            .arg(&pattern)
            .invoke_async(&mut conn)
            .await;

        match result {
            Ok(deleted_count) => {
                debug!(
                    "Invalidated {} keys with prefix: {} (using Lua script)",
                    deleted_count, self.key_prefix
                );
            }
            Err(e) => {
                // 如果 Lua 脚本失败（可能是 Redis 集群不支持），回退到 SCAN+DEL
                error!(
                    "Lua script invalidate_all failed: {}, falling back to SCAN+DEL",
                    e
                );
                self.invalidate_all_fallback().await;
            }
        }
    }
}

impl RedisObjectCache {
    /// 回退方案：使用 SCAN+DEL 模式删除
    async fn invalidate_all_fallback(&self) {
        let mut conn = self.connection.clone();
        let pattern = format!("{}*", self.key_prefix);
        let mut deleted_count = 0u64;
        let mut cursor: u64 = 0;

        loop {
            let scan_result: redis::RedisResult<(u64, Vec<String>)> = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await;

            match scan_result {
                Ok((new_cursor, keys)) => {
                    if !keys.is_empty() {
                        match conn.del::<&[String], u64>(&keys).await {
                            Ok(actual_deleted) => {
                                deleted_count += actual_deleted;
                            }
                            Err(e) => {
                                error!("Failed to delete keys during invalidate_all: {}", e);
                            }
                        }
                    }
                    cursor = new_cursor;
                    if cursor == 0 {
                        break;
                    }
                }
                Err(e) => {
                    error!("SCAN failed during invalidate_all: {}", e);
                    break;
                }
            }
        }

        debug!(
            "Invalidated {} keys with prefix: {} (fallback)",
            deleted_count, self.key_prefix
        );
    }
}
