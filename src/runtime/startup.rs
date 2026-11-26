use crate::analytics::global::set_global_click_manager;
use crate::analytics::manager::ClickManager;
use crate::analytics::{ClickDetail, DataRetentionTask, RawClickEvent, RollupManager};
use crate::config::{get_runtime_config, init_runtime_config, keys};
use crate::services::{
    AnalyticsService, ConfigService, ForgeLinkCache, LinkCache, LinkService, UserAgentStore,
    get_user_agent_store, set_global_user_agent_store,
};
use crate::storage::{SeaOrmStorage, StorageFactory};
use anyhow::{Context, Result};
use chrono::Utc;
use crossbeam_channel::Receiver;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::metrics::MetricsRecorder;

/// 服务器启动期组装好的全部运行时组件，由 [`prepare_server_startup`]
/// 一次性构建并交给 HTTP server 使用。
pub struct StartupContext {
    /// 进程级单实例守护句柄；持有期间阻止第二个实例同时运行。
    pub process_guard: crate::system::platform::ProcessGuard,
    /// 数据库存储后端。
    pub storage: Arc<SeaOrmStorage>,
    /// 链接缓存（Bloom/负缓存/对象缓存组合）。
    pub cache: Arc<dyn LinkCache>,
    /// 链接业务服务，共享给 HTTP 管理 API 与 IPC/CLI。
    pub link_service: Arc<LinkService>,
    /// 点击分析查询服务。
    pub analytics_service: Arc<AnalyticsService>,
    /// 运行时配置管理服务。
    pub config_service: Arc<ConfigService>,
    /// 路由前缀等路由相关配置。
    pub route_config: RouteConfig,
    /// 指标记录器；`metrics` feature 关闭时为 no-op 实现。
    pub metrics: Arc<dyn MetricsRecorder>,
    /// Prometheus 导出 provider；`metrics` feature 开启且 Forge 管线启动成功
    /// 时为 `Some`，供 HTTP 装配注入 `/metrics` 端点。
    pub metrics_provider: Option<aster_forge_metrics::MetricsProvider>,
    /// 点击事件聚合管理器；分析功能关闭时为 `None`。
    pub click_manager: Option<Arc<ClickManager>>,
    /// 原始点击事件的接收端；分析功能关闭时为 `None`，
    /// 由独立写线程消费以避免阻塞重定向热路径。
    pub raw_event_receiver: Option<Receiver<RawClickEvent>>,
    /// 数据保留策略后台任务；分析功能关闭时为 `None`。
    pub retention_task: Option<Arc<DataRetentionTask>>,
}

/// HTTP 路由前缀配置
#[derive(Clone, Debug)]
pub struct RouteConfig {
    /// 管理 API 的 URL 前缀（含开头斜杠）。
    pub admin_prefix: String,
    /// 健康检查端点的 URL 前缀（含开头斜杠）。
    pub health_prefix: String,
    /// 前端静态资源的 URL 前缀（含开头斜杠）。
    pub frontend_prefix: String,
    /// 是否挂载内置管理面板前端路由。
    pub enable_frontend: bool,
}

/// 准备服务器启动的上下文
/// 包括存储、缓存和路由配置等
pub async fn prepare_server_startup() -> Result<StartupContext> {
    let start_time = std::time::Instant::now();
    debug!("Starting pre-startup processing...");

    let process_guard = crate::system::platform::ProcessGuard::acquire()
        .context("Failed to acquire process guard")?;

    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|e| anyhow::anyhow!("Failed to install rustls crypto provider: {:?}", e))?;

    let metrics_runtime = crate::metrics::create_metrics_runtime();
    let metrics = metrics_runtime.recorder.clone();

    let storage = StorageFactory::create(metrics.clone())
        .await
        .context("Failed to create storage backend")?;
    info!(
        "Using storage backend: {}",
        storage.get_backend_config().storage_type
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

    // 初始化点击计数器（从 RuntimeConfig 读取配置）
    let (click_manager, raw_event_receiver) = init_click_tracking(&storage, &metrics);

    // 初始化缓存

    // 初始化缓存
    let cache = ForgeLinkCache::create(metrics.clone(), storage.clone())
        .await
        .context("Failed to create cache")?;

    // 加载短码到 Bloom Filter（内部自行从 DB 加载，不加载完整数据到 Object Cache）
    cache
        .rebuild_all()
        .await
        .context("Failed to initialize bloom filter")?;
    debug!("Bloom filter initialized");

    let reload_coordinator: Arc<dyn crate::system::reload::ReloadCoordinator> = Arc::new(
        crate::system::reload::DefaultReloadCoordinator::new(cache.clone()),
    );
    crate::system::reload::init_reload_coordinator(reload_coordinator);
    debug!("ReloadCoordinator initialized");

    // Create LinkService for unified link management
    let link_service = Arc::new(LinkService::new(storage.clone(), cache.clone()));

    // Create AnalyticsService for analytics queries
    let analytics_service = Arc::new(AnalyticsService::new(storage.clone()));

    // Create ConfigService for runtime configuration management
    let config_service = Arc::new(ConfigService::new().context("Failed to create ConfigService")?);

    // 初始化数据清理后台任务
    let rt = get_runtime_config();
    let enable_auto_rollup = rt.get_bool_or(keys::ANALYTICS_ENABLE_AUTO_ROLLUP, true);
    let retention_task = if enable_auto_rollup {
        let rollup_manager = Arc::new(RollupManager::new(storage.clone()));
        let retention_task = Arc::new(DataRetentionTask::new(
            storage.clone(),
            rollup_manager.clone(),
        ));
        debug!("Data retention background task initialized");
        Some(retention_task)
    } else {
        debug!("Auto rollup and data retention is disabled");
        None
    };

    // Initialize IPC handler with LinkService
    crate::system::ipc::handler::init_link_service(link_service.clone());

    // Initialize IPC handler with ConfigService
    crate::system::ipc::handler::init_config_service(config_service.clone());

    // Initialize IPC start time. The runtime task group owns the server loop.
    crate::system::ipc::handler::init_start_time();

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
        process_guard,
        storage,
        cache,
        link_service,
        analytics_service,
        config_service,
        route_config,
        metrics,
        metrics_provider: metrics_runtime.provider,
        click_manager,
        raw_event_receiver,
        retention_task,
    })
}

/// 初始化点击计数器（从 RuntimeConfig 读取配置），
/// 返回 (ClickManager, 原始事件接收端)；分析功能关闭时两者为 `None`。
fn init_click_tracking(
    storage: &Arc<SeaOrmStorage>,
    metrics: &Arc<dyn MetricsRecorder>,
) -> (Option<Arc<ClickManager>>, Option<Receiver<RawClickEvent>>) {
    let rt = get_runtime_config();
    let enable_click_tracking = rt.get_bool_or(keys::CLICK_ENABLE_TRACKING, true);
    let flush_interval = rt.get_u64_or(keys::CLICK_FLUSH_INTERVAL, 30);
    let max_clicks_before_flush = rt.get_u64_or(keys::CLICK_MAX_CLICKS_BEFORE_FLUSH, 100);

    if !enable_click_tracking {
        warn!("Click tracking is disabled in configuration");
        return (None, None);
    }

    let Some(sink) = storage.as_click_sink() else {
        warn!("Click sink is not available, ClickManager will not be initialized");
        return (None, None);
    };

    // 检查是否启用详细日志
    let enable_detailed_logging = rt.get_bool_or(keys::ANALYTICS_ENABLE_DETAILED_LOGGING, false);

    let (manager, raw_event_receiver) = if enable_detailed_logging {
        // SeaOrmStorage 实现了 DetailedClickSink trait
        let detailed_sink: Arc<dyn crate::analytics::DetailedClickSink> = storage.clone();
        info!("Detailed click logging enabled, initializing with DetailedClickSink");
        let (manager, rx) = ClickManager::with_detailed_logging(
            sink,
            detailed_sink,
            Duration::from_secs(flush_interval),
            usize::try_from(max_clicks_before_flush).unwrap_or(usize::MAX),
            metrics.clone(),
        );
        (manager, Some(rx))
    } else {
        let manager = ClickManager::new(
            sink,
            Duration::from_secs(flush_interval),
            usize::try_from(max_clicks_before_flush).unwrap_or(usize::MAX),
            metrics.clone(),
        );
        (manager, None)
    };

    let mgr = Arc::new(manager);
    set_global_click_manager(mgr.clone());

    debug!(
        "ClickManager initialized with {} seconds and {} max clicks before flush",
        flush_interval, max_clicks_before_flush
    );

    (Some(mgr), raw_event_receiver)
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
        info!(
            "Admin API is disabled. Run 'shortlinker reset-password' to set a password and enable it."
        );
    } else {
        info!("Admin API available at: {}", route_config.admin_prefix);
    }

    // 检查 Health API 是否启用
    let health_token = rt.get_or(keys::API_HEALTH_TOKEN, "");
    if health_token.is_empty() && admin_token.is_empty() {
        info!("Health API is disabled (api.health_token not set and ADMIN_TOKEN is empty)");
    } else {
        info!("Health API available at: {}", route_config.health_prefix);
    }

    // 检查前端路由是否启用，如果 api.admin_token 未设置 或者 features.enable_admin_panel 未设置为 true
    if !route_config.enable_frontend || admin_token.is_empty() {
        // 前端路由未启用
        info!(
            "Frontend routes are disabled (features.enable_admin_panel is false or api.admin_token not set)"
        );
    } else {
        // 检测自定义前端
        let custom_frontend = std::path::Path::new("./frontend-panel");
        if custom_frontend.exists() && custom_frontend.is_dir() {
            info!("Custom frontend detected at: ./frontend-panel");
        }
        info!(
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

/// 将原始点击事件转换为详细点击信息
pub(crate) fn process_raw_click_event(event: RawClickEvent) -> ClickDetail {
    // 在消费者端读取配置（不在热路径读取）
    let rt = get_runtime_config();
    let enable_ip_logging = rt.get_bool_or(keys::ANALYTICS_ENABLE_IP_LOGGING, true);

    // derive_source: utm_source > ref:{domain} > direct
    let source = Some(derive_source_from_raw(
        event.query.as_ref(),
        event.referrer.as_ref(),
    ));

    // UA hash
    let user_agent_hash = event
        .user_agent
        .as_ref()
        .and_then(|ua| get_user_agent_store().map(|store| store.get_or_create_hash(ua)));

    let ip_address = if enable_ip_logging { event.ip } else { None };

    ClickDetail {
        code: event.code,
        timestamp: Utc::now(),
        referrer: event.referrer,
        user_agent_hash,
        ip_address,
        country: None, // GeoIP 查询暂不在 channel 处理器中做
        city: None,
        source,
    }
}

/// 从原始数据推导流量来源
#[inline]
fn derive_source_from_raw(query: Option<&String>, referrer: Option<&String>) -> String {
    // 1. 检查 utm_source 参数
    if let Some(query) = query
        && let Some(utm_source) = extract_query_param(query, "utm_source")
    {
        return utm_source.into_owned();
    }

    // 2. 有 Referer → ref:{domain}
    if let Some(referer_url) = referrer
        && let Some(domain) = extract_domain(referer_url)
    {
        return format!("ref:{}", domain);
    }

    // 3. direct
    "direct".to_string()
}

/// 提取 URL 中的指定查询参数值
#[inline]
fn extract_query_param<'a>(query: &'a str, key: &str) -> Option<std::borrow::Cow<'a, str>> {
    let prefix = format!("{}=", key);
    for part in query.split('&') {
        if let Some(value) = part.strip_prefix(&prefix) {
            return urlencoding::decode(value).ok();
        }
    }
    None
}

/// 从 URL 提取域名
#[inline]
fn extract_domain(url: &str) -> Option<&str> {
    url.strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .and_then(|rest| rest.split('/').next())
        .and_then(|host| host.split(':').next())
        .filter(|s| !s.is_empty())
}
