use crate::cache::{self, CompositeCacheTrait};
use crate::storages::click::global::set_global_click_manager;
use crate::storages::click::manager::ClickManager;
use crate::storages::{Storage, StorageFactory};
use crate::system;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, warn};

pub struct StartupContext {
    pub storage: Arc<dyn Storage>,
    pub cache: Arc<dyn CompositeCacheTrait>,
    pub route_config: RouteConfig,
}

#[derive(Clone, Debug)]
pub struct RouteConfig {
    pub admin_prefix: String,
    pub health_prefix: String,
    pub frontend_prefix: String,
    pub enable_frontend: bool,
}

/// CLI 模式预处理
pub async fn cli_pre_startup() {
    // CLI Mode
    env::set_var("CLI_MODE", "true");
}

/// 准备服务器启动的上下文
/// 包括存储、缓存和路由配置等
pub async fn prepare_server_startup() -> StartupContext {
    std::env::set_var("CLI_MODE", "false");

    crate::system::lockfile::init_lockfile().expect("Failed to initialize lockfile");

    if cfg!(debug_assertions) {
        crate::storages::register::debug_storage_registry();
        crate::cache::register::debug_cache_registry();
    }

    let storage = StorageFactory::create()
        .await
        .expect("Failed to create storage backend");
    warn!(
        "Using storage backend: {}",
        storage.get_backend_config().await.storage_type
    );

    // 初始化点击计数器
    if let Some(sink) = storage.as_click_sink() {
        let mgr = Arc::new(ClickManager::new(sink, Duration::from_secs(30)));
        set_global_click_manager(mgr.clone());
        mgr.clone().start();
        debug!("ClickManager initialized with a 30 seconds flush interval");
    } else {
        warn!("Click sink is not available, ClickManager will not be initialized");
    }

    // 初始化缓存
    let cache = cache::CompositeCache::create(storage.preferred_cache().clone())
        .await
        .expect("Failed to create cache");

    let links = storage.load_all().await;
    cache
        .reconfigure(cache::traits::BloomConfig {
            capacity: links.len(),
            fp_rate: 0.001,
        })
        .await;
    cache.load_cache(links.clone()).await;
    debug!("L1/L2 cache initialized with {} links", links.len());

    system::setup_reload_mechanism(cache.clone(), storage.clone());

    // 提取路由配置
    let route_config = RouteConfig {
        admin_prefix: env::var("ADMIN_ROUTE_PREFIX").unwrap_or_else(|_| "/admin".to_string()),
        health_prefix: env::var("HEALTH_ROUTE_PREFIX").unwrap_or_else(|_| "/health".to_string()),
        frontend_prefix: env::var("FRONTEND_ROUTE_PREFIX").unwrap_or_else(|_| "/panel".to_string()),
        enable_frontend: env::var("ENABLE_ADMIN_PANEL")
            .map(|v| v == "true")
            .unwrap_or(false),
    };

    check_compoment_enabled(&route_config);

    StartupContext {
        storage,
        cache,
        route_config,
    }
}

fn check_compoment_enabled(route_config: &RouteConfig) {
    // 检查 Admin API 是否启用
    let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| "".to_string());
    if admin_token.is_empty() {
        warn!("Admin API is disabled (ADMIN_TOKEN not set)");
    } else {
        warn!("Admin API available at: {}", route_config.admin_prefix);
    }

    // 检查 Health API 是否启用
    let health_token = env::var("HEALTH_TOKEN").unwrap_or_default();
    if health_token.is_empty() && admin_token.is_empty() {
        warn!("Health API is disabled (HEALTH_TOKEN not set and ADMIN_TOKEN is empty)");
    } else {
        warn!("Health API available at: {}", route_config.health_prefix);
    }

    // 检查前端路由是否启用，如果 ADMIN_TOKEN 未设置 或者 ENABLE_ADMIN_PANEL 未设置为 true
    if !route_config.enable_frontend || admin_token.is_empty() {
        // 前端路由未启用
        warn!("Frontend routes are disabled (ENABLE_ADMIN_PANEL is false or ADMIN_TOKEN not set)");
    } else {
        warn!(
            "Frontend routes available at: {}",
            route_config.frontend_prefix
        );
    }
}
