//! Client module integration tests
//!
//! Tests for LinkClient (get_link, get_stats, list_links, batch_delete, import/export)
//! and ConfigClient (get_all, get, set, reset).
//! These tests exercise the fallback path (local service) since no IPC server is running.

use shortlinker::client::{ConfigClient, LinkClient, ServiceContext};
use shortlinker::config::init_config;
use shortlinker::config::runtime_config::init_runtime_config;
use shortlinker::metrics_core::NoopMetrics;
use shortlinker::services::ImportLinkItemRich;
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

// =============================================================================
// Helper: create ImportLinkItemRich
// =============================================================================

fn make_import_item(code: &str, target: &str) -> ImportLinkItemRich {
    ImportLinkItemRich {
        code: code.into(),
        target: target.into(),
        created_at: chrono::Utc::now(),
        expires_at: None,
        password: None,
        click_count: 0,
        row_num: None,
    }
}

// =============================================================================
// LinkClient::list_links tests
// =============================================================================

#[tokio::test]
async fn test_list_links_empty() {
    let (client, _td) = create_test_link_client().await;
    let (links, total) = client.list_links(1, 10, None).await.unwrap();
    assert!(links.is_empty());
    assert_eq!(total, 0);
}

#[tokio::test]
async fn test_list_links_pagination() {
    let (client, _td) = create_test_link_client().await;
    for i in 0..5 {
        client
            .create_link(
                Some(format!("page-{}", i)),
                format!("https://example.com/{}", i),
                false,
                None,
                None,
            )
            .await
            .unwrap();
    }

    let (links, total) = client.list_links(1, 2, None).await.unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(total, 5);
}

#[tokio::test]
async fn test_list_links_with_search() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("alpha".into()),
            "https://example.com/alpha".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();
    client
        .create_link(
            Some("beta".into()),
            "https://example.com/beta".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let (links, total) = client
        .list_links(1, 10, Some("alpha".into()))
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(links.len(), 1);
    assert_eq!(links[0].code, "alpha");
}

#[tokio::test]
async fn test_list_links_page_beyond_total() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("only-one".into()),
            "https://example.com".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let (links, total) = client.list_links(999, 10, None).await.unwrap();
    assert!(links.is_empty());
    assert_eq!(total, 1);
}

// =============================================================================
// LinkClient::batch_delete tests
// =============================================================================

#[tokio::test]
async fn test_batch_delete_all_exist() {
    let (client, _td) = create_test_link_client().await;
    for code in &["bd1", "bd2", "bd3"] {
        client
            .create_link(
                Some(code.to_string()),
                "https://example.com".into(),
                false,
                None,
                None,
            )
            .await
            .unwrap();
    }

    let result = client
        .batch_delete(vec!["bd1".into(), "bd2".into(), "bd3".into()])
        .await
        .unwrap();
    assert_eq!(result.deleted.len(), 3);
    assert!(result.not_found.is_empty());
    assert!(result.errors.is_empty());
}

#[tokio::test]
async fn test_batch_delete_some_not_found() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("exists".into()),
            "https://example.com".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let result = client
        .batch_delete(vec!["exists".into(), "ghost".into()])
        .await
        .unwrap();
    assert_eq!(result.deleted.len(), 1);
    assert_eq!(result.not_found.len(), 1);
    assert_eq!(result.not_found[0], "ghost");
}

#[tokio::test]
async fn test_batch_delete_empty() {
    let (client, _td) = create_test_link_client().await;
    let result = client.batch_delete(vec![]).await.unwrap();
    assert!(result.deleted.is_empty());
    assert!(result.not_found.is_empty());
}

// =============================================================================
// LinkClient::import_links tests
// =============================================================================

#[tokio::test]
async fn test_import_links_basic() {
    let (client, _td) = create_test_link_client().await;
    let items = vec![
        make_import_item("imp1", "https://example.com/1"),
        make_import_item("imp2", "https://example.com/2"),
        make_import_item("imp3", "https://example.com/3"),
    ];

    let result = client.import_links(items, false).await.unwrap();
    assert_eq!(result.success_count, 3);
    assert_eq!(result.skipped_count, 0);
    assert!(result.failed_items.is_empty());

    let stats = client.get_stats().await.unwrap();
    assert_eq!(stats.total_links, 3);
}

#[tokio::test]
async fn test_import_links_skip_mode() {
    let (client, _td) = create_test_link_client().await;
    // Create an existing link first
    client
        .create_link(
            Some("dup".into()),
            "https://example.com/original".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    // Import with overwrite=false (skip mode)
    let items = vec![make_import_item("dup", "https://example.com/new")];
    let result = client.import_links(items, false).await.unwrap();
    assert_eq!(result.skipped_count, 1);
    assert_eq!(result.success_count, 0);

    // Original should remain unchanged
    let link = client.get_link("dup".into()).await.unwrap().unwrap();
    assert_eq!(link.target, "https://example.com/original");
}

#[tokio::test]
async fn test_import_links_overwrite_mode() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("ow".into()),
            "https://example.com/old".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let items = vec![make_import_item("ow", "https://example.com/replaced")];
    let result = client.import_links(items, true).await.unwrap();
    assert_eq!(result.success_count, 1);

    let link = client.get_link("ow".into()).await.unwrap().unwrap();
    assert_eq!(link.target, "https://example.com/replaced");
}

#[tokio::test]
async fn test_import_links_empty() {
    let (client, _td) = create_test_link_client().await;
    let result = client.import_links(vec![], false).await.unwrap();
    assert_eq!(result.success_count, 0);
    assert_eq!(result.skipped_count, 0);
    assert!(result.failed_items.is_empty());
}

#[tokio::test]
async fn test_import_links_with_password() {
    let (client, _td) = create_test_link_client().await;
    let mut item = make_import_item("pwd-imp", "https://example.com/secret");
    item.password = Some("$argon2id$v=19$m=19456,t=2,p=1$fake_salt$fake_hash".into());

    let result = client.import_links(vec![item], false).await.unwrap();
    assert_eq!(result.success_count, 1);

    let link = client.get_link("pwd-imp".into()).await.unwrap().unwrap();
    assert!(link.password.is_some());
}

// =============================================================================
// LinkClient::export_links tests
// =============================================================================

#[tokio::test]
async fn test_export_links_empty() {
    let (client, _td) = create_test_link_client().await;
    let links = client.export_links().await.unwrap();
    assert!(links.is_empty());
}

#[tokio::test]
async fn test_export_links_round_trip() {
    let (client, _td) = create_test_link_client().await;
    // Create some links
    for i in 0..3 {
        client
            .create_link(
                Some(format!("exp-{}", i)),
                format!("https://example.com/{}", i),
                false,
                None,
                None,
            )
            .await
            .unwrap();
    }

    let exported = client.export_links().await.unwrap();
    assert_eq!(exported.len(), 3);

    // Verify all codes are present
    let codes: Vec<&str> = exported.iter().map(|l| l.code.as_str()).collect();
    assert!(codes.contains(&"exp-0"));
    assert!(codes.contains(&"exp-1"));
    assert!(codes.contains(&"exp-2"));
}

// =============================================================================
// LinkClient edge cases
// =============================================================================

#[tokio::test]
async fn test_create_link_auto_generate_code() {
    let (client, _td) = create_test_link_client().await;
    let result = client
        .create_link(None, "https://example.com/auto".into(), false, None, None)
        .await
        .unwrap();
    assert!(result.generated_code);
    assert!(!result.link.code.is_empty());
}

#[tokio::test]
async fn test_create_link_duplicate_without_force() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("dup-no-f".into()),
            "https://example.com".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let result = client
        .create_link(
            Some("dup-no-f".into()),
            "https://example.com/2".into(),
            false,
            None,
            None,
        )
        .await;
    assert!(
        result.is_err(),
        "Expected error for duplicate without force"
    );
}

#[tokio::test]
async fn test_create_link_duplicate_with_force() {
    let (client, _td) = create_test_link_client().await;
    client
        .create_link(
            Some("dup-f".into()),
            "https://example.com/old".into(),
            false,
            None,
            None,
        )
        .await
        .unwrap();

    let result = client
        .create_link(
            Some("dup-f".into()),
            "https://example.com/new".into(),
            true,
            None,
            None,
        )
        .await
        .unwrap();
    assert_eq!(result.link.target, "https://example.com/new");
}

#[tokio::test]
async fn test_create_link_with_expiration() {
    let (client, _td) = create_test_link_client().await;
    let result = client
        .create_link(
            Some("exp-test".into()),
            "https://example.com".into(),
            false,
            Some("2099-12-31T23:59:59Z".into()),
            None,
        )
        .await
        .unwrap();
    assert!(result.link.expires_at.is_some());
}

#[tokio::test]
async fn test_create_link_with_password() {
    let (client, _td) = create_test_link_client().await;
    let result = client
        .create_link(
            Some("pw-test".into()),
            "https://example.com".into(),
            false,
            None,
            Some("secret123".into()),
        )
        .await
        .unwrap();
    assert!(result.link.password.is_some());
    // Password should be hashed, not plaintext
    assert_ne!(result.link.password.as_deref(), Some("secret123"));
}

// =============================================================================
// ConfigClient set/reset tests
// =============================================================================

#[tokio::test]
async fn test_config_client_set_valid() {
    let (client, _td) = create_test_config_client().await;
    let result = client
        .set("features.random_code_length".into(), "8".into())
        .await;
    assert!(result.is_ok(), "set failed: {:?}", result);
    let view = result.unwrap();
    assert_eq!(view.key, "features.random_code_length");
    assert_eq!(view.value, "8");
}

#[tokio::test]
async fn test_config_client_set_invalid_key() {
    let (client, _td) = create_test_config_client().await;
    let result = client.set("totally.bogus.key".into(), "value".into()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_config_client_reset() {
    let (client, _td) = create_test_config_client().await;
    // Set a non-default value
    client
        .set("features.random_code_length".into(), "12".into())
        .await
        .unwrap();

    // Reset to default
    let result = client.reset("features.random_code_length".into()).await;
    assert!(result.is_ok(), "reset failed: {:?}", result);
    let view = result.unwrap();
    assert_eq!(view.key, "features.random_code_length");
    // Default should be "6"
    assert_eq!(view.value, "6");
}

#[tokio::test]
async fn test_config_client_get_all_with_category_filter() {
    let (client, _td) = create_test_config_client().await;
    let all = client.get_all(None).await.unwrap();
    let features = client.get_all(Some("features".into())).await.unwrap();

    // Filtered list should be a subset
    assert!(features.len() <= all.len());
    assert!(!features.is_empty());
    // All keys should belong to the "features" category
    for item in &features {
        assert!(
            item.key.starts_with("features."),
            "Expected features.* key, got: {}",
            item.key
        );
    }
}

// =============================================================================
// Error propagation tests
// =============================================================================

#[tokio::test]
async fn test_service_error_propagates_through_client() {
    let (client, _td) = create_test_link_client().await;
    // Try to update a nonexistent link — should propagate as ClientError
    let result = client
        .update_link(
            "nonexistent".into(),
            "https://example.com".into(),
            None,
            None,
        )
        .await;
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(
        err_msg.contains("not found") || err_msg.contains("Not found"),
        "Expected 'not found' in error, got: {}",
        err_msg
    );
}
