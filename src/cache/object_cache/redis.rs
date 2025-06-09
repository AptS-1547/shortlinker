use crate::cache::traits::{CacheResult, ObjectCache};
use crate::declare_object_cache_plugin;
use crate::storages::{SerializableShortLink, ShortLink};
use async_trait::async_trait;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use std::collections::HashMap;
use std::env;

/// Redis based object cache implementation
///
/// Requires `REDIS_URL` environment variable, defaulting to `redis://127.0.0.1/`
declare_object_cache_plugin!("redis", RedisCache);

pub struct RedisCache {
    conn: ConnectionManager,
}

impl RedisCache {
    pub fn new() -> Self {
        let url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string());
        let client = redis::Client::open(url).expect("Failed to create Redis client");
        // initialization happens in synchronous context, block on async connection
        let conn = tokio::runtime::Handle::current()
            .block_on(client.get_tokio_connection_manager())
            .expect("Failed to create Redis connection manager");
        Self { conn }
    }

    fn to_serializable(link: &ShortLink) -> SerializableShortLink {
        SerializableShortLink {
            short_code: link.code.clone(),
            target_url: link.target.clone(),
            created_at: link.created_at.to_rfc3339(),
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            click: 0,
        }
    }

    fn from_serializable(link: SerializableShortLink) -> ShortLink {
        let created_at = chrono::DateTime::parse_from_rfc3339(&link.created_at)
            .unwrap_or_else(|_| chrono::Utc::now().into())
            .with_timezone(&chrono::Utc);
        let expires_at = link
            .expires_at
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));
        ShortLink {
            code: link.short_code,
            target: link.target_url,
            created_at,
            expires_at,
        }
    }
}

#[async_trait]
impl ObjectCache for RedisCache {
    async fn get(&self, key: &str) -> CacheResult {
        let mut conn = self.conn.clone();
        match conn.get::<_, Option<String>>(key).await {
            Ok(Some(data)) => match serde_json::from_str::<SerializableShortLink>(&data) {
                Ok(s) => CacheResult::Found(Self::from_serializable(s)),
                Err(_) => CacheResult::ExistsButNoValue,
            },
            Ok(None) => CacheResult::NotFound,
            Err(e) => {
                tracing::error!("Redis get error: {}", e);
                CacheResult::NotFound
            }
        }
    }

    async fn insert(&self, key: String, value: ShortLink) {
        let mut conn = self.conn.clone();
        if let Ok(json) = serde_json::to_string(&Self::to_serializable(&value)) {
            let _: redis::RedisResult<()> = conn.set(key, json).await;
        }
    }

    async fn remove(&self, key: &str) {
        let mut conn = self.conn.clone();
        let _: redis::RedisResult<()> = conn.del(key).await;
    }

    async fn invalidate_all(&self) {
        let mut conn = self.conn.clone();
        let _: redis::RedisResult<()> = redis::cmd("FLUSHDB").query_async(&mut conn).await;
    }

    async fn load_l2_cache(&self, keys: HashMap<String, ShortLink>) {
        let mut conn = self.conn.clone();
        let mut pipe = redis::pipe();
        for (k, v) in keys {
            if let Ok(json) = serde_json::to_string(&Self::to_serializable(&v)) {
                pipe.set(k, json).ignore();
            }
        }
        let _: redis::RedisResult<()> = pipe.query_async(&mut conn).await;
    }
}
