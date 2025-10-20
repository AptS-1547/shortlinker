use crate::cache::{self, CompositeCacheTrait};
use crate::storages::click::global::set_global_click_manager;
use crate::storages::click::manager::ClickManager;
use crate::storages::{Storage, StorageFactory};
use crate::system::app_config::get_config;
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

/// CLI / TUI 模式预处理
#[cfg(any(feature = "cli", feature = "tui"))]
pub async fn cli_tui_pre_startup() {
    // CLI / TUI Mode
}

/// 准备服务器启动的上下文
/// 包括存储、缓存和路由配置等
pub async fn prepare_server_startup() -> StartupContext {
    crate::system::platform::init_lockfile().expect("Failed to initialize lockfile");

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

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
    // TODO: 点击上限刷新
    let config = get_config();
    if config.click_manager.enable_click_tracking {
        if let Some(sink) = storage.as_click_sink() {
            let mgr = Arc::new(ClickManager::new(
                sink,
                Duration::from_secs(config.click_manager.flush_interval),
                config.click_manager.max_clicks_before_flush as usize,
            ));
            set_global_click_manager(mgr.clone());

            // 启动后台任务，并保持强引用以确保任务不会被过早销毁
            let mgr_for_task = mgr.clone();
            tokio::spawn(async move {
                mgr_for_task.start_background_task().await;
            });

            debug!(
                "ClickManager initialized with {} seconds and {} max clicks before flush",
                config.click_manager.flush_interval, config.click_manager.max_clicks_before_flush
            );
        } else {
            warn!("Click sink is not available, ClickManager will not be initialized");
        }
    } else {
        warn!("Click tracking is disabled in configuration");
    }

    // 初始化缓存
    let cache = cache::CompositeCache::create()
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

    #[cfg(any(feature = "cli", feature = "tui"))]
    crate::system::platform::setup_reload_mechanism(cache.clone(), storage.clone()).await;

    // 提取路由配置
    let config = get_config();
    let route_config = RouteConfig {
        admin_prefix: config.routes.admin_prefix.clone(),
        health_prefix: config.routes.health_prefix.clone(),
        frontend_prefix: config.routes.frontend_prefix.clone(),
        enable_frontend: config.features.enable_admin_panel,
    };

    check_compoment_enabled(&route_config);

    StartupContext {
        storage,
        cache,
        route_config,
    }
}

fn check_compoment_enabled(route_config: &RouteConfig) {
    let config = get_config();
    // 检查 Admin API 是否启用
    let admin_token = &config.api.admin_token;
    if admin_token.is_empty() {
        warn!("Admin API is disabled (ADMIN_TOKEN not set)");
    } else {
        warn!("Admin API available at: {}", route_config.admin_prefix);
    }

    // 检查 Health API 是否启用
    let health_token = &config.api.health_token;
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
