//! Admin API integration tests
//!
//! Tests for the admin HTTP API endpoints (link CRUD, batch ops, stats).
//! Replaces the old services_tests.rs.disabled.

use std::collections::HashMap;
use std::sync::Arc;

use actix_web::http::StatusCode;
use actix_web::test::{self, TestRequest};
use actix_web::{web, App};
use async_trait::async_trait;
use serde_json::json;

use shortlinker::api::services::admin::routes::links_routes;
use shortlinker::api::services::admin::routes::stats_routes;
use shortlinker::api::services::admin::{ApiResponse, LinkResponse, PaginatedResponse, PostNewLink};
use shortlinker::cache::traits::{BloomConfig, CacheResult, CompositeCacheTrait};
use shortlinker::config::init_config;
use shortlinker::metrics_core::NoopMetrics;
use shortlinker::services::LinkService;
use shortlinker::storage::backend::SeaOrmStorage;
use shortlinker::storage::ShortLink;

use std::sync::Once;
use tempfile::TempDir;
use tokio::sync::RwLock;

// =============================================================================
// Test Setup
// =============================================================================

static INIT: Once = Once::new();
static TEST_DIR: std::sync::OnceLock<TempDir> = std::sync::OnceLock::new();
static SERVICE: std::sync::OnceLock<Arc<LinkService>> = std::sync::OnceLock::new();
static ADMIN_INIT: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

fn init_static_config() {
    INIT.call_once(|| {
        init_config();
    });
}

async fn init_admin_test_env() {
    init_static_config();

    ADMIN_INIT
        .get_or_init(|| async {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let db_path = temp_dir.path().join("admin_api_test.db");
            let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

            let storage = Arc::new(
                SeaOrmStorage::new(&db_url, "sqlite", NoopMetrics::arc())
                    .await
                    .expect("Failed to create storage"),
            );

            let cache: Arc<dyn CompositeCacheTrait> = Arc::new(MockCache::new());
            let service = Arc::new(LinkService::new(storage, cache));

            let _ = SERVICE.set(service);
            let _ = TEST_DIR.set(temp_dir);
        })
        .await;
}

fn get_service() -> Arc<LinkService> {
    SERVICE.get().expect("Service not initialized").clone()
}

/// Mock cache for admin API tests
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
    async fn remove(&self, key: &str) { self.data.write().await.remove(key); }
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
        for (k, v) in links { data.insert(k, v); }
    }
    async fn load_bloom(&self, _codes: &[String]) {}
    async fn reconfigure(&self, _config: BloomConfig) -> shortlinker::errors::Result<()> { Ok(()) }
    async fn bloom_check(&self, key: &str) -> bool { self.data.read().await.contains_key(key) }
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

/// Create a test app with admin link routes (no auth middleware)
macro_rules! admin_app {
    () => {{
        let service = get_service();
        test::init_service(
            App::new()
                .app_data(web::Data::new(service))
                .service(web::scope("/v1").service(links_routes()).service(stats_routes())),
        )
        .await
    }};
}

// =============================================================================
// Link CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_post_link_success() {
    init_admin_test_env().await;
    let app = admin_app!();

    let req = TestRequest::post()
        .uri("/v1/links")
        .set_json(json!({
            "code": "api-test1",
            "target": "https://example.com/api-test",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: ApiResponse<PostNewLink> = test::read_body_json(resp).await;
    assert_eq!(body.code, 0);
    let link = body.data.unwrap();
    assert_eq!(link.code.unwrap(), "api-test1");
    assert_eq!(link.target, "https://example.com/api-test");
}

#[tokio::test]
async fn test_post_link_auto_generate_code() {
    init_admin_test_env().await;
    let app = admin_app!();

    let req = TestRequest::post()
        .uri("/v1/links")
        .set_json(json!({
            "target": "https://example.com/auto-gen",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: ApiResponse<PostNewLink> = test::read_body_json(resp).await;
    assert_eq!(body.code, 0);
    let link = body.data.unwrap();
    assert!(link.code.is_some());
    assert_eq!(link.target, "https://example.com/auto-gen");
}

#[tokio::test]
async fn test_get_link_success() {
    init_admin_test_env().await;
    let app = admin_app!();

    // Create first
    let req = TestRequest::post()
        .uri("/v1/links")
        .set_json(json!({
            "code": "api-get1",
            "target": "https://example.com/get-test",
        }))
        .to_request();
    test::call_service(&app, req).await;

    // Get it
    let req = TestRequest::get().uri("/v1/links/api-get1").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: ApiResponse<LinkResponse> = test::read_body_json(resp).await;
    let link = body.data.unwrap();
    assert_eq!(link.code, "api-get1");
}

#[tokio::test]
async fn test_get_link_not_found() {
    init_admin_test_env().await;
    let app = admin_app!();

    let req = TestRequest::get()
        .uri("/v1/links/nonexistent-api-link")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_link_success() {
    init_admin_test_env().await;
    let app = admin_app!();

    // Create
    let req = TestRequest::post()
        .uri("/v1/links")
        .set_json(json!({
            "code": "api-upd1",
            "target": "https://example.com/old",
            "force": true,
        }))
        .to_request();
    test::call_service(&app, req).await;

    // Update
    let req = TestRequest::put()
        .uri("/v1/links/api-upd1")
        .set_json(json!({
            "target": "https://example.com/new",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: ApiResponse<PostNewLink> = test::read_body_json(resp).await;
    let link = body.data.unwrap();
    assert_eq!(link.target, "https://example.com/new");
}

#[tokio::test]
async fn test_delete_link_success() {
    init_admin_test_env().await;
    let app = admin_app!();

    // Create
    let req = TestRequest::post()
        .uri("/v1/links")
        .set_json(json!({
            "code": "api-del1",
            "target": "https://example.com/delete-me",
            "force": true,
        }))
        .to_request();
    test::call_service(&app, req).await;

    // Delete
    let req = TestRequest::delete()
        .uri("/v1/links/api-del1")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);

    // Verify deleted
    let req = TestRequest::get().uri("/v1/links/api-del1").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

// =============================================================================
// List / Stats Tests
// =============================================================================

#[tokio::test]
async fn test_get_all_links() {
    init_admin_test_env().await;
    let app = admin_app!();

    // Create a few links
    for i in 0..3 {
        let req = TestRequest::post()
            .uri("/v1/links")
            .set_json(json!({
                "code": format!("api-list{}", i),
                "target": format!("https://example.com/list{}", i),
                "force": true,
            }))
            .to_request();
        test::call_service(&app, req).await;
    }

    let req = TestRequest::get()
        .uri("/v1/links?page=1&page_size=10")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
    let body: PaginatedResponse<Vec<LinkResponse>> = test::read_body_json(resp).await;
    assert_eq!(body.code, 0);
    assert!(body.pagination.total >= 3);
    assert!(!body.data.unwrap().is_empty());
}

#[tokio::test]
async fn test_get_stats() {
    init_admin_test_env().await;
    let app = admin_app!();

    let req = TestRequest::get().uri("/v1/stats").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

// =============================================================================
// Batch Operations Tests
// =============================================================================

#[tokio::test]
async fn test_batch_create_links() {
    init_admin_test_env().await;
    let app = admin_app!();

    let req = TestRequest::post()
        .uri("/v1/links/batch")
        .set_json(json!({
            "links": [
                { "code": "api-batch1", "target": "https://example.com/b1" },
                { "code": "api-batch2", "target": "https://example.com/b2" },
            ]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_batch_delete_links() {
    init_admin_test_env().await;
    let app = admin_app!();

    // Create links to delete
    for code in &["api-bdel1", "api-bdel2"] {
        let req = TestRequest::post()
            .uri("/v1/links")
            .set_json(json!({
                "code": code,
                "target": "https://example.com/batch-del",
                "force": true,
            }))
            .to_request();
        test::call_service(&app, req).await;
    }

    // Batch delete
    let req = TestRequest::delete()
        .uri("/v1/links/batch")
        .set_json(json!({
            "codes": ["api-bdel1", "api-bdel2"]
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}
