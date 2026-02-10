//! IPC end-to-end integration tests
//!
//! Tests the full IPC communication path: client → Unix socket → server → handler → response.
//! Unlike ipc_handler_tests.rs which tests handle_command() directly, these tests
//! exercise the real socket transport, protocol encoding/decoding, and server loop.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use shortlinker::cache::CacheHealthStatus;
use shortlinker::cache::traits::{BloomConfig, CacheResult, CompositeCacheTrait};
use shortlinker::config::{init_config, set_ipc_socket_override};
use shortlinker::metrics_core::NoopMetrics;
use shortlinker::services::LinkService;
use shortlinker::storage::ShortLink;
use shortlinker::storage::backend::SeaOrmStorage;
use shortlinker::system::ipc::handler::{init_link_service, init_start_time};
use shortlinker::system::ipc::server::start_ipc_server;
use shortlinker::system::ipc::types::{ImportLinkData, IpcCommand, IpcResponse};
use shortlinker::system::ipc::{export_links, is_server_running, send_command};

use std::sync::Once;
use tempfile::TempDir;
use tokio::sync::RwLock;

// =============================================================================
// Test Setup
// =============================================================================

static INIT: Once = Once::new();
static TEST_DIR: std::sync::OnceLock<TempDir> = std::sync::OnceLock::new();
static SERVER_RT: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();

/// Get or create the dedicated runtime for the IPC server.
/// This runtime lives for the entire test process, keeping the server alive.
fn get_server_runtime() -> &'static tokio::runtime::Runtime {
    SERVER_RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Failed to create server runtime")
    })
}

/// Initialize config, handler, and start the IPC server on a dedicated runtime.
///
/// The server runs on a separate tokio runtime (not the test's runtime) so it
/// survives across multiple `#[tokio::test]` functions. `is_server_running()`
/// uses synchronous I/O, which is safe here because it runs on a std thread
/// (inside `call_once`), not on a tokio worker thread.
fn init_all() {
    INIT.call_once(|| {
        init_config();

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("test_ipc.sock");
        set_ipc_socket_override(socket_path.to_string_lossy().to_string());

        let db_path = temp_dir.path().join("ipc_e2e_test.db");
        let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

        // Run async initialization on a separate thread to avoid
        // "cannot call block_on from async context" panic.
        let handle = std::thread::spawn(move || {
            let rt = get_server_runtime();
            rt.block_on(async {
                init_start_time();

                let storage = Arc::new(
                    SeaOrmStorage::new(&db_url, "sqlite", NoopMetrics::arc())
                        .await
                        .expect("Failed to create storage"),
                );
                let cache: Arc<dyn CompositeCacheTrait> = Arc::new(MockCache::new());
                let service = Arc::new(LinkService::new(storage, cache));
                init_link_service(service);

                start_ipc_server()
                    .await
                    .expect("Failed to start IPC server");
            });
        });
        handle.join().expect("Init thread panicked");

        // Wait for server to be ready (synchronous, on std thread — safe)
        for _ in 0..50 {
            if is_server_running() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        assert!(is_server_running(), "IPC server did not start in time");

        let _ = TEST_DIR.set(temp_dir);
    });
}

/// Mock cache for IPC integration tests
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
    async fn health_check(&self) -> CacheHealthStatus {
        CacheHealthStatus {
            status: "healthy".to_string(),
            cache_type: "mock".to_string(),
            bloom_filter_enabled: false,
            negative_cache_enabled: true,
            error: None,
        }
    }
}

// =============================================================================
// End-to-End IPC Tests
// =============================================================================

#[tokio::test]
async fn test_e2e_ping() {
    init_all();

    let resp = send_command(IpcCommand::Ping).await.expect("Ping failed");
    match resp {
        IpcResponse::Pong {
            version,
            uptime_secs,
        } => {
            assert!(!version.is_empty());
            assert!(uptime_secs < 3600);
        }
        other => panic!("Expected Pong, got {:?}", other),
    }
}

#[tokio::test]
async fn test_e2e_add_and_get_link() {
    init_all();

    // Add a link
    let resp = send_command(IpcCommand::AddLink {
        code: Some("e2e-link1".to_string()),
        target: "https://example.com/e2e".to_string(),
        force: true,
        expires_at: None,
        password: None,
    })
    .await
    .expect("AddLink failed");

    match &resp {
        IpcResponse::LinkCreated {
            link,
            generated_code,
        } => {
            assert_eq!(link.code, "e2e-link1");
            assert_eq!(link.target, "https://example.com/e2e");
            assert!(!generated_code);
        }
        other => panic!("Expected LinkCreated, got {:?}", other),
    }

    // Get the link back
    let resp = send_command(IpcCommand::GetLink {
        code: "e2e-link1".to_string(),
    })
    .await
    .expect("GetLink failed");

    match resp {
        IpcResponse::LinkFound { link } => {
            let link = link.expect("Link should exist");
            assert_eq!(link.code, "e2e-link1");
            assert_eq!(link.target, "https://example.com/e2e");
        }
        other => panic!("Expected LinkFound, got {:?}", other),
    }
}

#[tokio::test]
async fn test_e2e_update_link() {
    init_all();

    send_command(IpcCommand::AddLink {
        code: Some("e2e-upd1".to_string()),
        target: "https://example.com/old".to_string(),
        force: true,
        expires_at: None,
        password: None,
    })
    .await
    .expect("AddLink failed");

    let resp = send_command(IpcCommand::UpdateLink {
        code: "e2e-upd1".to_string(),
        target: "https://example.com/new".to_string(),
        expires_at: None,
        password: None,
    })
    .await
    .expect("UpdateLink failed");

    match resp {
        IpcResponse::LinkUpdated { link } => {
            assert_eq!(link.code, "e2e-upd1");
            assert_eq!(link.target, "https://example.com/new");
        }
        other => panic!("Expected LinkUpdated, got {:?}", other),
    }
}

#[tokio::test]
async fn test_e2e_remove_link() {
    init_all();

    send_command(IpcCommand::AddLink {
        code: Some("e2e-rm1".to_string()),
        target: "https://example.com/rm".to_string(),
        force: true,
        expires_at: None,
        password: None,
    })
    .await
    .expect("AddLink failed");

    let resp = send_command(IpcCommand::RemoveLink {
        code: "e2e-rm1".to_string(),
    })
    .await
    .expect("RemoveLink failed");

    match resp {
        IpcResponse::LinkDeleted { code } => assert_eq!(code, "e2e-rm1"),
        other => panic!("Expected LinkDeleted, got {:?}", other),
    }

    // Verify removed
    let resp = send_command(IpcCommand::GetLink {
        code: "e2e-rm1".to_string(),
    })
    .await
    .expect("GetLink failed");

    match resp {
        IpcResponse::LinkFound { link } => assert!(link.is_none()),
        other => panic!("Expected LinkFound with None, got {:?}", other),
    }
}

#[tokio::test]
async fn test_e2e_list_links() {
    init_all();

    for i in 0..3 {
        send_command(IpcCommand::AddLink {
            code: Some(format!("e2e-list{}", i)),
            target: format!("https://example.com/list{}", i),
            force: true,
            expires_at: None,
            password: None,
        })
        .await
        .expect("AddLink failed");
    }

    let resp = send_command(IpcCommand::ListLinks {
        page: 1,
        page_size: 10,
        search: None,
    })
    .await
    .expect("ListLinks failed");

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
async fn test_e2e_import_export() {
    init_all();

    let links = vec![
        ImportLinkData {
            code: "e2e-imp1".to_string(),
            target: "https://example.com/imp1".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            expires_at: None,
            password: None,
            click_count: 0,
        },
        ImportLinkData {
            code: "e2e-imp2".to_string(),
            target: "https://example.com/imp2".to_string(),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            expires_at: None,
            password: None,
            click_count: 0,
        },
    ];

    let resp = send_command(IpcCommand::ImportLinks {
        links,
        overwrite: false,
    })
    .await
    .expect("ImportLinks failed");

    match resp {
        IpcResponse::ImportResult {
            success, failed, ..
        } => {
            assert_eq!(success, 2);
            assert_eq!(failed, 0);
        }
        other => panic!("Expected ImportResult, got {:?}", other),
    }

    let links = export_links()
        .await
        .expect("ExportLinks (streaming) failed");

    assert!(!links.is_empty());
    assert!(links.iter().any(|l| l.code == "e2e-imp1"));
}

#[tokio::test]
async fn test_e2e_stats() {
    init_all();

    let resp = send_command(IpcCommand::GetLinkStats)
        .await
        .expect("GetLinkStats failed");

    match resp {
        IpcResponse::StatsResult { total_clicks, .. } => {
            assert!(total_clicks >= 0);
        }
        other => panic!("Expected StatsResult, got {:?}", other),
    }
}

#[tokio::test]
async fn test_e2e_concurrent_commands() {
    init_all();

    let handles: Vec<_> = (0..5)
        .map(|i| {
            tokio::spawn(async move {
                send_command(IpcCommand::AddLink {
                    code: Some(format!("e2e-conc{}", i)),
                    target: format!("https://example.com/conc{}", i),
                    force: true,
                    expires_at: None,
                    password: None,
                })
                .await
            })
        })
        .collect();

    for handle in handles {
        let resp = handle
            .await
            .expect("Task panicked")
            .expect("Command failed");
        assert!(matches!(resp, IpcResponse::LinkCreated { .. }));
    }
}

#[tokio::test]
async fn test_e2e_shutdown_ack() {
    init_all();

    let resp = send_command(IpcCommand::Shutdown)
        .await
        .expect("Shutdown failed");

    assert!(matches!(resp, IpcResponse::ShuttingDown));
}
