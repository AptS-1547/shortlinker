use crate::analytics::global::set_global_click_manager;
use crate::analytics::manager::ClickManager;
use crate::analytics::{DataRetentionTask, RollupManager};
use crate::cache::{self, CompositeCacheTrait};
use crate::config::{get_runtime_config, init_runtime_config, keys};
use crate::services::{AnalyticsService, LinkService, UserAgentStore, set_global_user_agent_store};
use crate::storage::{SeaOrmStorage, StorageFactory};
use anyhow::{Context, Result};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

pub struct StartupContext {
    pub storage: Arc<SeaOrmStorage>,
    pub cache: Arc<dyn CompositeCacheTrait>,
    pub link_service: Arc<LinkService>,
    pub analytics_service: Arc<AnalyticsService>,
    pub route_config: RouteConfig,
}

#[derive(Clone, Debug)]
pub struct RouteConfig {
    pub admin_prefix: String,
    pub health_prefix: String,
    pub frontend_prefix: String,
    pub enable_frontend: bool,
}

/// CLI / TUI 模式预处理（预留扩展点）
///
/// 当前为空实现，供未来 CLI/TUI 特定初始化使用。
#[cfg(any(feature = "cli", feature = "tui"))]
pub async fn cli_tui_pre_startup() {
    // Reserved for future CLI/TUI-specific initialization
}

/// 准备服务器启动的上下文
/// 包括存储、缓存和路由配置等
pub async fn prepare_server_startup() -> Result<StartupContext> {
    let start_time = std::time::Instant::now();
    debug!("Starting pre-startup processing...");

    crate::system::platform::init_lockfile().context("Failed to initialize lockfile")?;

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| anyhow::anyhow!("Failed to install rustls crypto provider: {:?}", e))?;

    let storage = StorageFactory::create()
        .await
        .context("Failed to create storage backend")?;
    warn!(
        "Using storage backend: {}",
        storage.get_backend_config().await.storage_type
    );

    // 初始化运行时配置系统
    let db = storage.get_db().clone();
    init_runtime_config(db.clone())
        .await
        .context("Failed to initialize runtime config")?;
    debug!("Runtime config system initialized");

    // 初始化 UserAgentStore（UA 去重存储）
    let ua_store = UserAgentStore::new();
    if let Err(e) = ua_store.load_known_hashes(&db).await {
        warn!("Failed to preload UserAgent hashes (non-fatal): {}", e);
    }

    let known_count = ua_store.known_count();
    set_global_user_agent_store(ua_store);
    debug!(
        "UserAgentStore initialized with {} known hashes",
        known_count
    );

    // 启动 UserAgent 后台刷新任务（每 30 秒批量写入新 UA）
    let db_for_ua = storage.get_db().clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(30)).await;
            if let Some(store) = crate::services::get_user_agent_store()
                && let Err(e) = store.flush_pending(&db_for_ua).await
            {
                tracing::warn!("Failed to flush UserAgent pending inserts: {}", e);
            }
        }
    });

    // 初始化点击计数器（从 RuntimeConfig 读取配置）
    let rt = get_runtime_config();
    let enable_click_tracking = rt.get_bool_or(keys::CLICK_ENABLE_TRACKING, true);
    let flush_interval = rt.get_u64_or(keys::CLICK_FLUSH_INTERVAL, 30);
    let max_clicks_before_flush = rt.get_u64_or(keys::CLICK_MAX_CLICKS_BEFORE_FLUSH, 100);

    if enable_click_tracking {
        if let Some(sink) = storage.as_click_sink() {
            // 检查是否启用详细日志
            let enable_detailed_logging =
                rt.get_bool_or(keys::ANALYTICS_ENABLE_DETAILED_LOGGING, false);

            let mgr = if enable_detailed_logging {
                // SeaOrmStorage 实现了 DetailedClickSink trait
                let detailed_sink: Arc<dyn crate::analytics::DetailedClickSink> = storage.clone();
                info!("Detailed click logging enabled, initializing with DetailedClickSink");
                Arc::new(ClickManager::with_detailed_logging(
                    sink,
                    detailed_sink,
                    Duration::from_secs(flush_interval),
                    max_clicks_before_flush as usize,
                ))
            } else {
                Arc::new(ClickManager::new(
                    sink,
                    Duration::from_secs(flush_interval),
                    max_clicks_before_flush as usize,
                ))
            };

            set_global_click_manager(mgr.clone());

            // 启动后台任务，并保持强引用以确保任务不会被过早销毁
            let mgr_for_task = mgr.clone();
            tokio::spawn(async move {
                mgr_for_task.start_background_task().await;
            });

            debug!(
                "ClickManager initialized with {} seconds and {} max clicks before flush",
                flush_interval, max_clicks_before_flush
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
        .context("Failed to create cache")?;

    // 只加载短码到 Bloom Filter（不加载完整数据到 Object Cache）
    let codes = storage
        .load_all_codes()
        .await
        .context("Failed to load codes for bloom filter")?;
    let codes_count = codes.len();
    cache
        .reconfigure(cache::traits::BloomConfig {
            capacity: codes_count,
            fp_rate: 0.001,
        })
        .await
        .context("Failed to reconfigure cache")?;
    cache.load_bloom(&codes).await;
    debug!("Bloom filter initialized with {} codes", codes_count);

    // Initialize the ReloadCoordinator (must be before setup_reload_mechanism)
    crate::system::reload::init_default_coordinator(cache.clone(), storage.clone());
    debug!("ReloadCoordinator initialized");

    // Create LinkService for unified link management
    let link_service = Arc::new(LinkService::new(storage.clone(), cache.clone()));

    // Create AnalyticsService for analytics queries
    let analytics_service = Arc::new(AnalyticsService::new(storage.clone()));

    // 初始化数据清理后台任务
    let enable_auto_rollup = rt.get_bool_or(keys::ANALYTICS_ENABLE_AUTO_ROLLUP, true);
    if enable_auto_rollup {
        let rollup_manager = Arc::new(RollupManager::new(storage.clone()));
        let retention_task = Arc::new(DataRetentionTask::new(
            storage.clone(),
            rollup_manager.clone(),
        ));
        // 每 4 小时运行一次清理
        retention_task.spawn_background_task(4);
        debug!("Data retention background task initialized");
    } else {
        debug!("Auto rollup and data retention is disabled");
    }

    // Initialize IPC handler with LinkService
    crate::system::ipc::handler::init_link_service(link_service.clone());

    // Initialize IPC start time and start IPC server
    crate::system::ipc::handler::init_start_time();
    #[cfg(any(feature = "cli", feature = "tui"))]
    {
        let config = crate::config::get_config();
        if config.ipc.enabled {
            crate::system::ipc::server::start_ipc_server().await;
            debug!(
                "IPC server started on {}",
                config.ipc.effective_socket_path()
            );
        } else {
            warn!("IPC server is disabled by configuration");
        }
    }

    // 提取路由配置（从 RuntimeConfig 读取）
    let rt = get_runtime_config();
    let route_config = RouteConfig {
        admin_prefix: rt.get_or(keys::ROUTES_ADMIN_PREFIX, "/admin"),
        health_prefix: rt.get_or(keys::ROUTES_HEALTH_PREFIX, "/health"),
        frontend_prefix: rt.get_or(keys::ROUTES_FRONTEND_PREFIX, "/panel"),
        enable_frontend: rt.get_bool_or(keys::FEATURES_ENABLE_ADMIN_PANEL, false),
    };

    check_component_enabled(&route_config);

    debug!(
        "Pre-startup processing completed in {} ms",
        start_time.elapsed().as_millis()
    );

    Ok(StartupContext {
        storage,
        cache,
        link_service,
        analytics_service,
        route_config,
    })
}

fn check_component_enabled(route_config: &RouteConfig) {
    let rt = get_runtime_config();

    // 检查 JWT Secret 安全性
    check_jwt_secret_security(rt);

    // 检查 Cookie Secure 标志
    if !rt.get_bool_or(keys::API_COOKIE_SECURE, true) {
        warn!(
            "WARNING: Cookie Secure flag is disabled. \
            Cookies will be sent over unencrypted HTTP connections. \
            Enable cookie_secure=true for production environments."
        );
    }

    // 检查 Admin API 是否启用
    let admin_token = rt.get_or(keys::API_ADMIN_TOKEN, "");
    if admin_token.is_empty() {
        warn!("Admin API is disabled (ADMIN_TOKEN not set)");
    } else {
        warn!("Admin API available at: {}", route_config.admin_prefix);
    }

    // 检查 Health API 是否启用
    let health_token = rt.get_or(keys::API_HEALTH_TOKEN, "");
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
        // 检测自定义前端
        let custom_frontend = std::path::Path::new("./frontend-panel");
        if custom_frontend.exists() && custom_frontend.is_dir() {
            info!("Custom frontend detected at: ./frontend-panel");
        }
        warn!(
            "Frontend routes available at: {}",
            route_config.frontend_prefix
        );
    }
}

/// 检查 JWT Secret 安全性
fn check_jwt_secret_security(rt: &crate::config::RuntimeConfig) {
    let jwt_secret = rt.get_or(keys::API_JWT_SECRET, "");
    // 检查 JWT Secret 长度
    if jwt_secret.len() < 32 {
        warn!(
            "WARNING: JWT Secret is too short ({} bytes). \
            Recommended minimum is 32 bytes for security.",
            jwt_secret.len()
        );
    }

    // 检查 Admin Token 长度
    let admin_token = rt.get_or(keys::API_ADMIN_TOKEN, "");
    if !admin_token.is_empty() && admin_token.len() < 8 {
        warn!("WARNING: Admin Token is very short. Consider using a stronger token.");
    }
}
