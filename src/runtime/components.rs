//! Server mode
//!
//! This module contains the HTTP server startup logic.
//! It configures and starts the HTTP server with all necessary routes.

use actix_cors::Cors;
use actix_web::{
    App, HttpServer,
    middleware::{Compress, Condition, DefaultHeaders},
    web,
};
use anyhow::{Result, anyhow};
use aster_forge_middleware::actix::metrics::MetricsMiddleware;
use aster_forge_middleware::actix::request_id::RequestIdMiddleware;
use aster_forge_runtime::{RuntimeComponentKind, RuntimeServiceComponent};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::api::middleware::{AdminAuth, CsrfGuard, FrontendGuard, HealthAuth};
use crate::api::services::{
    AppStartTime, admin::routes::admin_v1_routes, frontend_routes, health_routes, redirect_routes,
};
use crate::config::{HttpMethod, get_runtime_config, keys};
use crate::runtime::startup::StartupContext;
use crate::services::GeoIpProvider;

/// CORS configuration loaded from RuntimeConfig
#[derive(Clone, Debug)]
struct CorsSettings {
    enabled: bool,
    allowed_origins: Vec<String>,
    allowed_methods: Vec<HttpMethod>,
    allowed_headers: Vec<String>,
    max_age: u64,
    allow_credentials: bool,
}

impl CorsSettings {
    fn from_runtime_config() -> Self {
        let rt = get_runtime_config();

        // 解析 JSON 配置
        let allowed_origins: Vec<String> = rt.get_json_or(keys::CORS_ALLOWED_ORIGINS, Vec::new());
        let allowed_methods: Vec<HttpMethod> =
            rt.get_json_or(keys::CORS_ALLOWED_METHODS, Vec::new());
        let allowed_headers: Vec<String> = rt.get_json_or(
            keys::CORS_ALLOWED_HEADERS,
            vec![
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "Accept".to_string(),
                "X-CSRF-Token".to_string(),
            ],
        );

        Self {
            enabled: rt.get_bool_or(keys::CORS_ENABLED, false),
            allowed_origins,
            allowed_methods,
            allowed_headers,
            max_age: rt.get_u64_or(keys::CORS_MAX_AGE, 3600),
            allow_credentials: rt.get_bool_or(keys::CORS_ALLOW_CREDENTIALS, false),
        }
    }
}

/// Validate CORS configuration at startup (runs once)
fn validate_cors_config(cors_config: &CorsSettings) {
    if !cors_config.enabled {
        return;
    }

    if cors_config.allowed_origins.is_empty() {
        warn!(
            "CORS enabled but allowed_origins is empty. \
            No cross-origin requests will be allowed. \
            Set allowed_origins explicitly or use '[\"*\"]' for any origin."
        );
    }

    let is_any_origin = cors_config.allowed_origins.iter().any(|o| o == "*");
    if is_any_origin && cors_config.allow_credentials {
        tracing::error!(
            "SECURITY WARNING: allow_any_origin + allow_credentials is a dangerous combination! \
            Any website can make authenticated cross-origin requests. \
            Disabling credentials for safety."
        );
    }
}

/// Build CORS middleware from configuration (only called when CORS is enabled)
fn build_cors_middleware(cors_config: &CorsSettings) -> Cors {
    let mut cors = Cors::default();

    // Track if we're using wildcard origins
    let is_any_origin = cors_config.allowed_origins.iter().any(|o| o == "*");

    // Configure allowed origins
    if cors_config.allowed_origins.is_empty() {
        // Empty origins = same-origin only (no cross-origin requests allowed)
        // Don't call allow_any_origin(), use default same-origin policy
    } else if is_any_origin {
        cors = cors.allow_any_origin();
    } else {
        for origin in &cors_config.allowed_origins {
            cors = cors.allowed_origin(origin);
        }
    }

    // Configure allowed methods
    let methods: Vec<actix_web::http::Method> = cors_config
        .allowed_methods
        .iter()
        .filter_map(|m| m.to_string().parse().ok())
        .collect();
    if !methods.is_empty() {
        cors = cors.allowed_methods(methods);
    }

    // Configure allowed headers
    for header in &cors_config.allowed_headers {
        cors = cors.allowed_header(header);
    }

    // Configure max age
    cors = cors.max_age(usize::try_from(cors_config.max_age).unwrap_or(usize::MAX));

    // Configure credentials with security check
    // Disallow any_origin + credentials combination as it's a security vulnerability
    // (actix-cors echoes Origin header instead of *, which browsers accept)
    if is_any_origin && cors_config.allow_credentials {
        // Don't call supports_credentials() - force disable for security
    } else if cors_config.allow_credentials {
        cors = cors.supports_credentials();
    }

    cors
}

/// 检查并记录代理检测模式 + Unix Socket 模式
fn log_client_ip_mode(config: &crate::config::StaticConfig, rt: &crate::config::RuntimeConfig) {
    // Check and log proxy detection mode + Unix Socket mode
    #[cfg(unix)]
    let is_tcp_mode = if let Some(ref socket_path) = config.server.unix_socket {
        info!(
            "Unix Socket mode enabled: {}. \
             Local proxy transport is trusted; configure the proxy to set \
             X-Forwarded-For for per-client login rate limiting and analytics.",
            socket_path
        );
        false
    } else {
        true
    };
    #[cfg(not(unix))]
    let is_tcp_mode = true;

    if !is_tcp_mode {
        return;
    }

    let trusted_proxies_json = rt.get_or(keys::API_TRUSTED_PROXIES, "[]");
    let trusted_proxies: Vec<String> = match serde_json::from_str(&trusted_proxies_json) {
        Ok(v) => v,
        Err(e) => {
            warn!(
                "Invalid JSON for trusted_proxies '{}': {}, using empty list",
                trusted_proxies_json, e
            );
            Vec::new()
        }
    };
    if trusted_proxies.is_empty() {
        info!(
            "Client IP extraction: direct peer mode enabled. \
             X-Forwarded-For is ignored until api.trusted_proxies is configured."
        );
    } else {
        info!(
            "Client IP extraction: explicit trusted proxies configured: {:?}",
            trusted_proxies
        );
    }
}

/// 组装产品路由：admin / health / 前端静态资源 / 重定向
fn product_routes(
    admin_prefix: &str,
    health_prefix: &str,
    frontend_prefix: &str,
) -> actix_web::Scope {
    web::scope("")
        .service(
            web::scope(admin_prefix)
                .wrap(CsrfGuard)
                .wrap(AdminAuth)
                .service(admin_v1_routes()),
        )
        .service(
            web::scope(health_prefix)
                .wrap(HealthAuth)
                .service(health_routes()),
        )
        .service(
            web::scope(frontend_prefix)
                .wrap(FrontendGuard)
                .service(frontend_routes()),
        )
        .service(redirect_routes())
}

/// 加载并校验 CORS 配置，同时记录代理检测 / Unix Socket 模式
fn init_http_runtime_config(
    config: &crate::config::StaticConfig,
    rt: &crate::config::RuntimeConfig,
) -> CorsSettings {
    // Load CORS configuration from RuntimeConfig
    let cors_config = CorsSettings::from_runtime_config();

    // Validate CORS configuration at startup (runs once, not per worker)
    validate_cors_config(&cors_config);

    // Check and log proxy detection mode + Unix Socket mode
    log_client_ip_mode(config, rt);

    cors_config
}

/// Run the HTTP server
///
/// This function:
/// 1. Records startup time
/// 2. Prepares server components (cache, storage, routes)
/// 3. Configures and starts the HTTP server
/// 4. Listens for graceful shutdown signals
///
/// **Note**: Logging system must be initialized before calling this function
pub fn http_component(
    startup: &StartupContext,
    shutdown_token: CancellationToken,
) -> Result<RuntimeServiceComponent<actix_web::dev::Server>> {
    // Record application start time
    let app_start_time = AppStartTime {
        start_datetime: chrono::Utc::now(),
    };

    let cache = startup.cache.clone();
    let storage = startup.storage.clone();
    let link_service = startup.link_service.clone();
    let analytics_service = startup.analytics_service.clone();
    let config_service = startup.config_service.clone();
    let route = startup.route_config.clone();
    let metrics = startup.metrics.clone();

    let admin_prefix = route.admin_prefix;
    let health_prefix = route.health_prefix;
    let frontend_prefix = route.frontend_prefix;

    // Load configuration (after database sync in prepare_server_startup)
    let config = crate::config::get_config();

    let cpu_count = config.server.cpu_count.min(32);
    info!("Using {} CPU cores for the server", cpu_count);

    // GeoIP provider is startup-config driven and can be toggled at runtime via
    // `analytics.enable_geo_lookup` (runtime config). We always initialize it here so
    // toggling doesn't require a restart; actual lookup only happens when enabled.
    let geoip_provider = Arc::new(GeoIpProvider::new(&config.analytics));

    let forge_metrics = metrics.forge_recorder();

    // Load + validate CORS configuration and log proxy/socket mode (runs once, not per worker)
    let cors_config = init_http_runtime_config(&config, get_runtime_config());

    // Configure HTTP server
    let metrics_for_server = metrics.clone();
    let forge_metrics_for_server = forge_metrics.clone();
    #[cfg(feature = "metrics")]
    let metrics_provider_for_server = startup.metrics_provider.clone();
    let server = HttpServer::new(move || {
        // Build CORS middleware (Condition::new skips it entirely when disabled)
        let cors = build_cors_middleware(&cors_config);
        let cors_enabled = cors_config.enabled;

        let app = App::new()
            .wrap(MetricsMiddleware)
            .wrap(RequestIdMiddleware) // 为每个请求生成 request_id
            .wrap(Condition::new(cors_enabled, cors))
            .wrap(Compress::default())
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(storage.clone()))
            .app_data(web::Data::new(link_service.clone()))
            .app_data(web::Data::new(analytics_service.clone()))
            .app_data(web::Data::new(config_service.clone()))
            .app_data(web::Data::new(geoip_provider.clone()))
            .app_data(web::Data::new(app_start_time.clone()))
            .app_data(web::PayloadConfig::new(1024 * 1024))
            .app_data(web::Data::new(metrics_for_server.clone()))
            .app_data(web::Data::<dyn aster_forge_metrics::MetricsRecorder>::from(
                forge_metrics_for_server.clone(),
            ));

        // Prometheus export provider is only present with the `metrics`
        // feature; the observability resource reads it from app data.
        #[cfg(feature = "metrics")]
        let app = if let Some(provider) = metrics_provider_for_server.as_ref() {
            app.app_data(web::Data::new(provider.clone()))
        } else {
            app
        };

        app.wrap(aster_forge_middleware::actix::security_headers::default_headers())
            .wrap(
                DefaultHeaders::new()
                    .add(("Connection", "keep-alive"))
                    .add(("Keep-Alive", "timeout=30, max=1000"))
                    .add(("Cache-Control", "no-cache, no-store, must-revalidate")),
            )
            .service(product_routes(
                &admin_prefix,
                &health_prefix,
                &frontend_prefix,
            ))
    })
    .disable_signals()
    .keep_alive(std::time::Duration::from_secs(30))
    .client_request_timeout(std::time::Duration::from_secs(5))
    .client_disconnect_timeout(std::time::Duration::from_secs(1))
    .workers(cpu_count);

    // Bind to Unix socket or TCP address
    let server = {
        #[cfg(unix)]
        {
            if let Some(ref socket_path) = config.server.unix_socket {
                info!("Starting server on Unix socket: {}", socket_path);
                if std::path::Path::new(socket_path).exists() {
                    std::fs::remove_file(socket_path)?;
                }
                Some(server.bind_uds(socket_path)?)
            } else {
                let bind_address = format!("{}:{}", config.server.host, config.server.port);
                info!("Starting server at http://{}", bind_address);
                Some(server.bind(bind_address)?)
            }
        }

        #[cfg(not(unix))]
        {
            let bind_address = format!("{}:{}", config.server.host, config.server.port);
            info!("Starting server at http://{}", bind_address);
            Some(server.bind(bind_address)?)
        }
    }
    .ok_or_else(|| anyhow!("Server binding failed"))?
    .run();
    let server_handle = server.handle();

    Ok(RuntimeServiceComponent::new(
        "http",
        RuntimeComponentKind::Product,
        server,
        shutdown_token,
        move || async move {
            server_handle.stop(true).await;
        },
    ))
}
