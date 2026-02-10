//! Client module integration tests
//!
//! Tests for LinkClient (get_link, get_stats) and ServiceContext lazy initialization.
//! These tests exercise the fallback path (local service) since no IPC server is running.

use shortlinker::client::{ConfigClient, LinkClient, ServiceContext};
use shortlinker::config::init_config;
use shortlinker::config::runtime_config::init_runtime_config;
use shortlinker::metrics_core::NoopMetrics;
use shortlinker::storage::backend::{SeaOrmStorage, connect_sqlite, run_migrations};
use std::sync::{Arc, Once};
use tempfile::TempDir;

// =============================================================================
// Global initialization (shared across all tests in this file)
// =============================================================================

static INIT: Once = Once::new();
static TEST_DIR: std::sync::OnceLock<TempDir> = std::sync::OnceLock::new();
static RT_INIT: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

fn init_static_config() {
    INIT.call_once(|| {
        init_config();
    });
}

async fn init_test_runtime_config() {
    init_static_config();
    RT_INIT
        .get_or_init(|| async {
            let td = TempDir::new().unwrap();
            let p = td.path().join("client_rt.db");
            let u = format!("sqlite://{}?mode=rwc", p.display());
            let db = connect_sqlite(&u).await.unwrap();
            run_migrations(&db).await.unwrap();
            init_runtime_config(db).await.unwrap();
            let _ = TEST_DIR.set(td);
        })
        .await;
}

/// Create a LinkClient backed by a fresh temporary SQLite database.
async fn create_test_link_client() -> (LinkClient, TempDir) {
    init_test_runtime_config().await;
    let td = TempDir::new().unwrap();
    let p = td.path().join("client_test.db");
    let u = format!("sqlite://{}?mode=rwc", p.display());
    let s = SeaOrmStorage::new(&u, "sqlite", NoopMetrics::arc())
        .await
        .unwrap();
    let storage = Arc::new(s);
    let ctx = Arc::new(ServiceContext::with_storage(storage));
    let client = LinkClient::new(ctx);
    (client, td)
}

/// Create a ConfigClient backed by a fresh temporary SQLite database.
async fn create_test_config_client() -> (ConfigClient, TempDir) {
    init_test_runtime_config().await;
    let td = TempDir::new().unwrap();
    let p = td.path().join("cfg_client_test.db");
    let u = format!("sqlite://{}?mode=rwc", p.display());
    let s = SeaOrmStorage::new(&u, "sqlite", NoopMetrics::arc())
        .await
        .unwrap();
    let storage = Arc::new(s);
    let ctx = Arc::new(ServiceContext::with_storage(storage));
    let client = ConfigClient::new(ctx);
    (client, td)
}

// =============================================================================
// LinkClient::get_link tests
// =============================================================================

#[tokio::test]
async fn test_get_link_found() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("get-test".into()),
            "https://example.com".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let result = client.get_link("get-test".into()).await.unwrap();
    assert!(result.is_some());
    let link = result.unwrap();
    assert_eq!(link.code, "get-test");
    assert_eq!(link.target, "https://example.com");
}

#[tokio::test]
async fn test_get_link_not_found() {
    let (client, _td) = create_test_link_client().await;
    let result = client.get_link("nonexistent".into()).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_link_returns_correct_target() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("target-check".into()),
            "https://rust-lang.org".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let link = client
        .get_link("target-check".into())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(link.target, "https://rust-lang.org");
}

// =============================================================================
// LinkClient::get_stats tests
// =============================================================================

#[tokio::test]
async fn test_get_stats_empty() {
    let (client, _td) = create_test_link_client().await;
    let stats = client.get_stats().await.unwrap();
    assert_eq!(stats.total_links, 0);
    assert_eq!(stats.active_links, 0);
}

#[tokio::test]
async fn test_get_stats_with_links() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("stats1".into()),
            "https://example.com/1".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();
    client
        .create_link(
            Some("stats2".into()),
            "https://example.com/2".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let stats = client.get_stats().await.unwrap();
    assert_eq!(stats.total_links, 2);
    assert_eq!(stats.active_links, 2);
}

#[tokio::test]
async fn test_get_stats_total_clicks_starts_at_zero() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("clicks-test".into()),
            "https://example.com".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let stats = client.get_stats().await.unwrap();
    assert_eq!(stats.total_clicks, 0);
}

// =============================================================================
// ServiceContext tests
// =============================================================================

#[tokio::test]
async fn test_service_context_with_storage_reuses_link_service() {
    init_test_runtime_config().await;
    let td = TempDir::new().unwrap();
    let p = td.path().join("ctx_test.db");
    let u = format!("sqlite://{}?mode=rwc", p.display());
    let s = SeaOrmStorage::new(&u, "sqlite", NoopMetrics::arc())
        .await
        .unwrap();
    let ctx = ServiceContext::with_storage(Arc::new(s));

    // First call initializes
    let svc1 = ctx.get_link_service().await;
    assert!(svc1.is_ok());

    // Second call returns same instance (OnceCell)
    let svc2 = ctx.get_link_service().await;
    assert!(svc2.is_ok());

    // Both should be the same Arc (pointer equality)
    assert!(Arc::ptr_eq(svc1.unwrap(), svc2.unwrap()));
}

#[tokio::test]
async fn test_service_context_config_service_initializes() {
    init_test_runtime_config().await;
    let td = TempDir::new().unwrap();
    let p = td.path().join("ctx_cfg_test.db");
    let u = format!("sqlite://{}?mode=rwc", p.display());
    let s = SeaOrmStorage::new(&u, "sqlite", NoopMetrics::arc())
        .await
        .unwrap();
    let ctx = ServiceContext::with_storage(Arc::new(s));

    let svc = ctx.get_config_service().await;
    assert!(svc.is_ok());
}

#[tokio::test]
async fn test_service_context_config_service_reuses_instance() {
    init_test_runtime_config().await;
    let td = TempDir::new().unwrap();
    let p = td.path().join("ctx_cfg_reuse.db");
    let u = format!("sqlite://{}?mode=rwc", p.display());
    let s = SeaOrmStorage::new(&u, "sqlite", NoopMetrics::arc())
        .await
        .unwrap();
    let ctx = ServiceContext::with_storage(Arc::new(s));

    let svc1 = ctx.get_config_service().await.unwrap();
    let svc2 = ctx.get_config_service().await.unwrap();
    assert!(Arc::ptr_eq(svc1, svc2));
}

// =============================================================================
// ConfigClient integration tests
// =============================================================================

#[tokio::test]
async fn test_config_client_get_all() {
    let (client, _td) = create_test_config_client().await;
    let result = client.get_all(None).await;
    assert!(result.is_ok(), "get_all failed: {:?}", result);
    let configs = result.unwrap();
    // Should return at least some config items (from definitions)
    assert!(!configs.is_empty());
}

#[tokio::test]
async fn test_config_client_get_known_key() {
    let (client, _td) = create_test_config_client().await;
    let result = client.get("features.random_code_length".into()).await;
    assert!(result.is_ok(), "get failed: {:?}", result);
    let view = result.unwrap();
    assert_eq!(view.key, "features.random_code_length");
}

#[tokio::test]
async fn test_config_client_get_unknown_key() {
    let (client, _td) = create_test_config_client().await;
    let result = client.get("nonexistent.key".into()).await;
    assert!(result.is_err());
}

// =============================================================================
// LinkClient combined operations tests
// =============================================================================

#[tokio::test]
async fn test_create_get_delete_lifecycle() {
    let (client, _td) = create_test_link_client().await;

    // Create
    client
        .create_link(
            Some("lifecycle".into()),
            "https://example.com/lifecycle".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    // Get - should exist
    let link = client.get_link("lifecycle".into()).await.unwrap();
    assert!(link.is_some());
    assert_eq!(link.unwrap().target, "https://example.com/lifecycle");

    // Delete
    client.delete_link("lifecycle".into()).await.unwrap();

    // Get - should not exist
    let link = client.get_link("lifecycle".into()).await.unwrap();
    assert!(link.is_none());

    // Stats should reflect deletion
    let stats = client.get_stats().await.unwrap();
    assert_eq!(stats.total_links, 0);
}

#[tokio::test]
async fn test_get_link_after_update() {
    let (client, _td) = create_test_link_client().await;

    client
        .create_link(
            Some("upd-get".into()),
            "https://example.com/old".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    client
        .update_link(
            "upd-get".into(),
            "https://example.com/new".into(),
            None,
            None,
        )
        .await
        .unwrap();

    let link = client.get_link("upd-get".into()).await.unwrap().unwrap();
    assert_eq!(link.target, "https://example.com/new");
}
