//! Redirect service tests
//!
//! Tests for the core URL redirect functionality.
//! This is the most critical path: short code → 307 redirect.

use std::collections::HashMap;
use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use actix_web::{web, App};
use async_trait::async_trait;
use chrono::Utc;

use shortlinker::api::services::redirect::redirect_routes;
use shortlinker::cache::traits::{BloomConfig, CacheResult, CompositeCacheTrait};
use shortlinker::config::init_config;
use shortlinker::config::runtime_config::init_runtime_config;
use shortlinker::metrics_core::{MetricsRecorder, NoopMetrics};
use shortlinker::storage::backend::{connect_sqlite, run_migrations, SeaOrmStorage};
use shortlinker::storage::ShortLink;

use std::sync::Once;
use tempfile::TempDir;
use tokio::sync::RwLock;

// =============================================================================
// Test Setup
// =============================================================================

static INIT: Once = Once::new();
static TEST_DIR: std::sync::OnceLock<TempDir> = std::sync::OnceLock::new();
static RT_INIT: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();
static STORAGE: std::sync::OnceLock<Arc<SeaOrmStorage>> = std::sync::OnceLock::new();

fn init_static_config() {
    INIT.call_once(|| {
        init_config();
    });
}

async fn init_test_env() {
    init_static_config();

    RT_INIT
        .get_or_init(|| async {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let db_path = temp_dir.path().join("redirect_test.db");
            let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

            let db = connect_sqlite(&db_url)
                .await
                .expect("Failed to connect to SQLite");
            run_migrations(&db.clone())
                .await
                .expect("Failed to run migrations");

            init_runtime_config(db)
                .await
                .expect("Failed to init runtime config");

            let storage = Arc::new(
                SeaOrmStorage::new(&db_url, "sqlite", NoopMetrics::arc())
                    .await
                    .expect("Failed to create storage"),
            );
            let _ = STORAGE.set(storage);
            let _ = TEST_DIR.set(temp_dir);
        })
        .await;
}

fn get_storage() -> Arc<SeaOrmStorage> {
    STORAGE.get().expect("Storage not initialized").clone()
}

/// Mock cache for redirect tests
struct MockCache {
    data: RwLock<HashMap<String, ShortLink>>,
    not_found: RwLock<std::collections::HashSet<String>>,
}

impl MockCache {
    fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            not_found: RwLock::new(std::collections::HashSet::new()),
        }
    }
}

#[async_trait]
impl CompositeCacheTrait for MockCache {
    async fn get(&self, key: &str) -> CacheResult {
        if self.not_found.read().await.contains(key) {
            return CacheResult::NotFound;
        }
        match self.data.read().await.get(key) {
            Some(link) => CacheResult::Found(link.clone()),
            None => CacheResult::Miss,
        }
    }

    async fn insert(&self, key: &str, value: ShortLink, _ttl_secs: Option<u64>) {
        self.not_found.write().await.remove(key);
        self.data.write().await.insert(key.to_string(), value);
    }

    async fn remove(&self, key: &str) {
        self.data.write().await.remove(key);
    }

    async fn invalidate_all(&self) {
        self.data.write().await.clear();
        self.not_found.write().await.clear();
    }

    async fn rebuild_all(&self) -> shortlinker::errors::Result<()> {
        self.data.write().await.clear();
        self.not_found.write().await.clear();
        Ok(())
    }

    async fn mark_not_found(&self, key: &str) {
        self.not_found.write().await.insert(key.to_string());
    }

    async fn load_cache(&self, links: HashMap<String, ShortLink>) {
        let mut data = self.data.write().await;
        for (k, v) in links {
            data.insert(k, v);
        }
    }

    async fn load_bloom(&self, _codes: &[String]) {}

    async fn reconfigure(&self, _config: BloomConfig) -> shortlinker::errors::Result<()> {
        Ok(())
    }

    async fn bloom_check(&self, key: &str) -> bool {
        self.data.read().await.contains_key(key)
    }

    async fn health_check(&self) -> shortlinker::cache::CacheHealthStatus {
        shortlinker::cache::CacheHealthStatus {
            status: "healthy".to_string(),
            cache_type: "mock".to_string(),
            bloom_filter_enabled: false,
            negative_cache_enabled: true,
            error: None,
        }
    }
}

/// Create a test app with redirect routes
macro_rules! redirect_app {
    ($cache:expr) => {{
        let storage = get_storage();
        let metrics: Arc<dyn MetricsRecorder> = NoopMetrics::arc();

        test::init_service(
            App::new()
                .app_data(web::Data::new($cache as Arc<dyn CompositeCacheTrait>))
                .app_data(web::Data::new(storage))
                .app_data(web::Data::new(metrics))
                .service(redirect_routes()),
        )
        .await
    }};
}

// =============================================================================
// Redirect Tests
// =============================================================================

#[tokio::test]
async fn test_redirect_existing_link_from_cache() {
    init_test_env().await;

    let cache = Arc::new(MockCache::new());
    // Pre-populate cache with a link
    cache
        .insert(
            "cached1",
            ShortLink {
                code: "cached1".to_string(),
                target: "https://example.com/cached".to_string(),
                created_at: Utc::now(),
                expires_at: None,
                password: None,
                click: 0,
            },
            Some(3600),
        )
        .await;

    let app = redirect_app!(cache);

    let req = TestRequest::get().uri("/cached1").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = resp.headers().get("Location").unwrap().to_str().unwrap();
    assert_eq!(location, "https://example.com/cached");
}

#[tokio::test]
async fn test_redirect_existing_link_from_db() {
    init_test_env().await;

    let storage = get_storage();
    // Insert a link directly into the database
    storage
        .set(ShortLink {
            code: "dblink1".to_string(),
            target: "https://example.com/fromdb".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            password: None,
            click: 0,
        })
        .await
        .expect("Failed to insert link");

    // Empty cache → cache miss → falls through to DB
    let cache = Arc::new(MockCache::new());
    let app = redirect_app!(cache);

    let req = TestRequest::get().uri("/dblink1").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
    let location = resp.headers().get("Location").unwrap().to_str().unwrap();
    assert_eq!(location, "https://example.com/fromdb");
}

#[tokio::test]
async fn test_redirect_nonexistent_link() {
    init_test_env().await;

    let cache = Arc::new(MockCache::new());
    let app = redirect_app!(cache);

    let req = TestRequest::get()
        .uri("/nonexistent-code")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_redirect_negative_cache_hit() {
    init_test_env().await;

    let cache = Arc::new(MockCache::new());
    // Mark as not found in cache
    cache.mark_not_found("negcached").await;

    let app = redirect_app!(cache);

    let req = TestRequest::get().uri("/negcached").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_redirect_invalid_short_code() {
    init_test_env().await;

    let cache = Arc::new(MockCache::new());
    let app = redirect_app!(cache);

    // Special characters that are not valid short codes
    let req = TestRequest::get().uri("/%3Cscript%3E").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_redirect_expired_link() {
    init_test_env().await;

    let storage = get_storage();
    // Insert an expired link
    storage
        .set(ShortLink {
            code: "expired1".to_string(),
            target: "https://example.com/expired".to_string(),
            created_at: Utc::now() - chrono::Duration::days(30),
            expires_at: Some(Utc::now() - chrono::Duration::days(1)),
            password: None,
            click: 0,
        })
        .await
        .expect("Failed to insert link");

    let cache = Arc::new(MockCache::new());
    let app = redirect_app!(cache);

    let req = TestRequest::get().uri("/expired1").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_redirect_head_request() {
    init_test_env().await;

    let cache = Arc::new(MockCache::new());
    cache
        .insert(
            "headtest",
            ShortLink {
                code: "headtest".to_string(),
                target: "https://example.com/head".to_string(),
                created_at: Utc::now(),
                expires_at: None,
                password: None,
                click: 0,
            },
            Some(3600),
        )
        .await;

    let app = redirect_app!(cache);

    let req = TestRequest::default()
        .method(actix_web::http::Method::HEAD)
        .uri("/headtest")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::TEMPORARY_REDIRECT);
}
