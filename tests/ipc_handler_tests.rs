//! IPC Handler tests
//!
//! Tests for the IPC command handler (`handle_command`).
//! Replaces the old system_tests.rs.disabled.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use shortlinker::cache::traits::{BloomConfig, CacheResult, CompositeCacheTrait};
use shortlinker::config::init_config;
use shortlinker::metrics_core::NoopMetrics;
use shortlinker::services::LinkService;
use shortlinker::storage::ShortLink;
use shortlinker::storage::backend::SeaOrmStorage;
use shortlinker::system::ipc::handler::{handle_command, init_link_service, init_start_time};
use shortlinker::system::ipc::types::{ImportLinkData, IpcCommand, IpcResponse};
use std::sync::Once;
use tempfile::TempDir;
use tokio::sync::RwLock;

// =============================================================================
// Test Setup
// =============================================================================

static INIT: Once = Once::new();

fn init_test_env() {
    INIT.call_once(|| {
        init_config();
        init_start_time();
    });
}

/// Mock cache for testing
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

/// Static TempDir to keep the database alive for the entire test process.
/// OnceLock<LINK_SERVICE> can only be set once, so all tests share one DB.
static IPC_TEST_DIR: std::sync::OnceLock<TempDir> = std::sync::OnceLock::new();
static IPC_INIT: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

/// Initialize the IPC handler once for all tests.
async fn setup_ipc_handler() {
    init_test_env();

    IPC_INIT
        .get_or_init(|| async {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let db_path = temp_dir.path().join("ipc_test.db");
            let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

            let storage = Arc::new(
                SeaOrmStorage::new(&db_url, "sqlite", NoopMetrics::arc())
                    .await
                    .expect("Failed to create storage"),
            );

            let cache: Arc<dyn CompositeCacheTrait> = Arc::new(MockCache::new());
            let service = Arc::new(LinkService::new(storage, cache));

            init_link_service(service);

            // Store TempDir in static to keep it alive
            let _ = IPC_TEST_DIR.set(temp_dir);
        })
        .await;
}

// =============================================================================
// Ping / Shutdown Tests
// =============================================================================

#[tokio::test]
async fn test_ping_command() {
    setup_ipc_handler().await;

    let resp = handle_command(IpcCommand::Ping).await;
    match resp {
        IpcResponse::Pong {
            version,
            uptime_secs,
        } => {
            assert!(!version.is_empty());
            // uptime should be very small in tests
            assert!(uptime_secs < 3600);
        }
        other => panic!("Expected Pong, got {:?}", other),
    }
}

#[tokio::test]
async fn test_shutdown_command() {
    setup_ipc_handler().await;

    let resp = handle_command(IpcCommand::Shutdown).await;
    assert!(matches!(resp, IpcResponse::ShuttingDown));
}

// =============================================================================
// Link CRUD Tests
// =============================================================================

#[tokio::test]
async fn test_add_link_command() {
    setup_ipc_handler().await;

    let resp = handle_command(IpcCommand::AddLink {
        code: Some("ipc-test1".to_string()),
        target: "https://example.com".to_string(),
        force: false,
        expires_at: None,
        password: None,
    })
    .await;

    match resp {
        IpcResponse::LinkCreated {
            link,
            generated_code,
        } => {
            assert_eq!(link.code, "ipc-test1");
            assert_eq!(link.target, "https://example.com");
            assert!(!generated_code);
        }
        other => panic!("Expected LinkCreated, got {:?}", other),
    }
}

#[tokio::test]
async fn test_add_link_auto_generate_code() {
    setup_ipc_handler().await;

    let resp = handle_command(IpcCommand::AddLink {
        code: None,
        target: "https://example.com/auto".to_string(),
        force: false,
        expires_at: None,
        password: None,
    })
    .await;

    match resp {
        IpcResponse::LinkCreated {
            link,
            generated_code,
        } => {
            assert!(!link.code.is_empty());
            assert_eq!(link.target, "https://example.com/auto");
            assert!(generated_code);
        }
        other => panic!("Expected LinkCreated, got {:?}", other),
    }
}

#[tokio::test]
async fn test_get_link_command() {
    setup_ipc_handler().await;

    // Create a link first
    handle_command(IpcCommand::AddLink {
        code: Some("ipc-get1".to_string()),
        target: "https://example.com/get".to_string(),
        force: true,
        expires_at: None,
        password: None,
    })
    .await;

    // Get it
    let resp = handle_command(IpcCommand::GetLink {
        code: "ipc-get1".to_string(),
    })
    .await;

    match resp {
        IpcResponse::LinkFound { link } => {
            let link = link.expect("Link should exist");
            assert_eq!(link.code, "ipc-get1");
            assert_eq!(link.target, "https://example.com/get");
        }
        other => panic!("Expected LinkFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_get_nonexistent_link() {
    setup_ipc_handler().await;

    let resp = handle_command(IpcCommand::GetLink {
        code: "nonexistent-ipc-link".to_string(),
    })
    .await;

    match resp {
        IpcResponse::LinkFound { link } => {
            assert!(link.is_none());
        }
        other => panic!("Expected LinkFound with None, got {:?}", other),
    }
}

#[tokio::test]
async fn test_update_link_command() {
    setup_ipc_handler().await;

    // Create
    handle_command(IpcCommand::AddLink {
        code: Some("ipc-upd1".to_string()),
        target: "https://example.com/old".to_string(),
        force: true,
        expires_at: None,
        password: None,
    })
    .await;

    // Update
    let resp = handle_command(IpcCommand::UpdateLink {
        code: "ipc-upd1".to_string(),
        target: "https://example.com/new".to_string(),
        expires_at: None,
        password: None,
    })
    .await;

    match resp {
        IpcResponse::LinkUpdated { link } => {
            assert_eq!(link.code, "ipc-upd1");
            assert_eq!(link.target, "https://example.com/new");
        }
        other => panic!("Expected LinkUpdated, got {:?}", other),
    }
}

#[tokio::test]
async fn test_remove_link_command() {
    setup_ipc_handler().await;

    // Create
    handle_command(IpcCommand::AddLink {
        code: Some("ipc-rm1".to_string()),
        target: "https://example.com/rm".to_string(),
        force: true,
        expires_at: None,
        password: None,
    })
    .await;

    // Remove
    let resp = handle_command(IpcCommand::RemoveLink {
        code: "ipc-rm1".to_string(),
    })
    .await;

    match resp {
        IpcResponse::LinkDeleted { code } => {
            assert_eq!(code, "ipc-rm1");
        }
        other => panic!("Expected LinkDeleted, got {:?}", other),
    }
}

#[tokio::test]
async fn test_remove_nonexistent_link() {
    setup_ipc_handler().await;

    let resp = handle_command(IpcCommand::RemoveLink {
        code: "nonexistent-ipc-rm".to_string(),
    })
    .await;

    // Should return an error since the link doesn't exist
    assert!(matches!(resp, IpcResponse::Error { .. }));
}

// =============================================================================
// List / Import / Export / Stats Tests
// =============================================================================

#[tokio::test]
async fn test_list_links_command() {
    setup_ipc_handler().await;

    // Create a few links
    for i in 0..3 {
        handle_command(IpcCommand::AddLink {
            code: Some(format!("ipc-list{}", i)),
            target: format!("https://example.com/list{}", i),
            force: true,
            expires_at: None,
            password: None,
        })
        .await;
    }

    let resp = handle_command(IpcCommand::ListLinks {
        page: 1,
        page_size: 10,
        search: None,
    })
    .await;

    match resp {
        IpcResponse::LinkList {
            links,
            total,
            page,
            page_size,
        } => {
            assert!(total >= 3);
            assert_eq!(page, 1);
            assert_eq!(page_size, 10);
            assert!(!links.is_empty());
        }
        other => panic!("Expected LinkList, got {:?}", other),
    }
}

#[tokio::test]
async fn test_list_links_with_search() {
    setup_ipc_handler().await;

    // Create a link with a unique code
    handle_command(IpcCommand::AddLink {
        code: Some("ipc-searchable-xyz".to_string()),
        target: "https://example.com/searchable".to_string(),
        force: true,
        expires_at: None,
        password: None,
    })
    .await;

    let resp = handle_command(IpcCommand::ListLinks {
        page: 1,
        page_size: 10,
        search: Some("searchable-xyz".to_string()),
    })
    .await;

    match resp {
        IpcResponse::LinkList { links, total, .. } => {
            assert!(total >= 1);
            assert!(links.iter().any(|l| l.code == "ipc-searchable-xyz"));
        }
        other => panic!("Expected LinkList, got {:?}", other),
    }
}

#[tokio::test]
async fn test_import_links_command() {
    setup_ipc_handler().await;

    let links = vec![
        ImportLinkData {
            code: "ipc-imp1".to_string(),
            target: "https://example.com/imp1".to_string(),
            expires_at: None,
            password: None,
        },
        ImportLinkData {
            code: "ipc-imp2".to_string(),
            target: "https://example.com/imp2".to_string(),
            expires_at: None,
            password: None,
        },
    ];

    let resp = handle_command(IpcCommand::ImportLinks {
        links,
        overwrite: false,
    })
    .await;

    match resp {
        IpcResponse::ImportResult {
            success, failed, ..
        } => {
            assert_eq!(success, 2);
            assert_eq!(failed, 0);
        }
        other => panic!("Expected ImportResult, got {:?}", other),
    }
}

#[tokio::test]
async fn test_export_links_command() {
    setup_ipc_handler().await;

    // Create a link to ensure there's something to export
    handle_command(IpcCommand::AddLink {
        code: Some("ipc-exp1".to_string()),
        target: "https://example.com/exp".to_string(),
        force: true,
        expires_at: None,
        password: None,
    })
    .await;

    let resp = handle_command(IpcCommand::ExportLinks).await;

    match resp {
        IpcResponse::ExportResult { links } => {
            assert!(!links.is_empty());
        }
        other => panic!("Expected ExportResult, got {:?}", other),
    }
}

#[tokio::test]
async fn test_get_stats_command() {
    setup_ipc_handler().await;

    let resp = handle_command(IpcCommand::GetLinkStats).await;

    match resp {
        IpcResponse::StatsResult { total_clicks, .. } => {
            // Stats should be non-negative
            assert!(total_clicks >= 0);
        }
        other => panic!("Expected StatsResult, got {:?}", other),
    }
}
