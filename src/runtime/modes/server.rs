//! Server mode
//!
//! This module contains the HTTP server startup logic.
//! It configures and starts the HTTP server with all necessary routes.

use actix_cors::Cors;
use actix_web::{
    App, HttpServer,
    middleware::{Compress, DefaultHeaders},
    web,
};
use anyhow::Result;
use std::sync::Arc;
use tracing::warn;

use crate::api::middleware::{
    AdminAuth, CsrfGuard, FrontendGuard, HealthAuth, RequestIdMiddleware, TimingMiddleware,
};
use crate::api::services::{
    AppStartTime, admin::routes::admin_v1_routes, frontend_routes, health_routes, redirect_routes,
};
use crate::config::{HttpMethod, get_runtime_config, keys};
use crate::runtime::lifetime;
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

/// Server configuration
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub server_host: String,
    pub server_port: u16,
    #[cfg(unix)]
    pub unix_socket_path: Option<String>,
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

/// Build CORS middleware from configuration
fn build_cors_middleware(cors_config: &CorsSettings) -> Cors {
    // When CORS is disabled, use browser's default same-origin policy (restrictive)
    if !cors_config.enabled {
        return Cors::default();
    }

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
    cors = cors.max_age(cors_config.max_age as usize);

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

/// Run the HTTP server
///
/// This function:
/// 1. Records startup time
/// 2. Prepares server components (cache, storage, routes)
/// 3. Configures and starts the HTTP server
/// 4. Listens for graceful shutdown signals
///
/// **Note**: Logging system must be initialized before calling this function
pub async fn run_server() -> Result<()> {
    // Record application start time
    let app_start_time = AppStartTime {
        start_datetime: chrono::Utc::now(),
    };

    // Prepare server startup (cache, storage, routes)
    let startup = lifetime::startup::prepare_server_startup()
        .await
        .map_err(|e| {
            tracing::error!("Server startup failed: {}", e);
            e
        })?;

    let cache = startup.cache.clone();
    let storage = startup.storage.clone();
    let link_service = startup.link_service.clone();
    let analytics_service = startup.analytics_service.clone();
    let route = startup.route_config.clone();

    let admin_prefix = route.admin_prefix;
    let health_prefix = route.health_prefix;
    let frontend_prefix = route.frontend_prefix;

    // Load configuration (after database sync in prepare_server_startup)
    let config = crate::config::get_config();
    let rt = get_runtime_config();

    // Load server configuration
    let server_config = ServerConfig {
        server_host: config.server.host.clone(),
        server_port: config.server.port,
        #[cfg(unix)]
        unix_socket_path: config.server.unix_socket.clone(),
    };

    let cpu_count = config.server.cpu_count.min(32);
    warn!("Using {} CPU cores for the server", cpu_count);

    // GeoIP provider is startup-config driven and can be toggled at runtime via
    // `analytics.enable_geo_lookup` (runtime config). We always initialize it here so
    // toggling doesn't require a restart; actual lookup only happens when enabled.
    let geoip_provider = Arc::new(GeoIpProvider::new(&config.analytics));

    // Load CORS configuration from RuntimeConfig
    let cors_config = CorsSettings::from_runtime_config();

    // Validate CORS configuration at startup (runs once, not per worker)
    validate_cors_config(&cors_config);

    // Check and log proxy detection mode + Unix Socket mode
    let mut is_tcp_mode = true;
    #[cfg(unix)]
    if let Some(ref socket_path) = config.server.unix_socket {
        warn!(
            "Unix Socket mode enabled: {}. \
             Rate limiting requires nginx to set X-Forwarded-For header.",
            socket_path
        );
        is_tcp_mode = false;
    }

    if is_tcp_mode {
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
            warn!(
                "Login rate limiting: Auto-detect mode enabled. \
                 Connections from private IPs will use X-Forwarded-For. \
                 To disable, configure api.trusted_proxies explicitly."
            );
        } else {
            warn!(
                "Login rate limiting: Explicit trusted proxies configured: {:?}",
                trusted_proxies
            );
        }
    }

    // Clone db reference before storage moves into HttpServer closure
    let db_for_shutdown = storage.get_db().clone();

    // Configure HTTP server
    let server = HttpServer::new(move || {
        // Build CORS middleware
        let cors = build_cors_middleware(&cors_config);

        App::new()
            .wrap(TimingMiddleware) // 最外层，记录请求延迟
            .wrap(RequestIdMiddleware) // 为每个请求生成 request_id
            .wrap(cors)
            .wrap(Compress::default())
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(storage.clone()))
            .app_data(web::Data::new(link_service.clone()))
            .app_data(web::Data::new(analytics_service.clone()))
            .app_data(web::Data::new(geoip_provider.clone()))
            .app_data(web::Data::new(app_start_time.clone()))
            .app_data(web::PayloadConfig::new(1024 * 1024))
            .wrap(
                DefaultHeaders::new()
                    .add(("Connection", "keep-alive"))
                    .add(("Keep-Alive", "timeout=30, max=1000"))
                    .add(("Cache-Control", "no-cache, no-store, must-revalidate")),
            )
            .service(
                web::scope(&admin_prefix)
                    .wrap(CsrfGuard)
                    .wrap(AdminAuth)
                    .service(admin_v1_routes()),
            )
            .service(
                web::scope(&health_prefix)
                    .wrap(HealthAuth)
                    .service(health_routes()),
            )
            .service(
                web::scope(&frontend_prefix)
                    .wrap(FrontendGuard)
                    .service(frontend_routes()),
            )
            .service(redirect_routes())
    })
    .keep_alive(std::time::Duration::from_secs(30))
    .client_request_timeout(std::time::Duration::from_millis(5000))
    .client_disconnect_timeout(std::time::Duration::from_millis(1000))
    .workers(cpu_count);

    // Bind to Unix socket or TCP address
    let server = {
        #[cfg(unix)]
        {
            if let Some(ref socket_path) = server_config.unix_socket_path {
                warn!("Starting server on Unix socket: {}", socket_path);
                if std::path::Path::new(socket_path).exists() {
                    std::fs::remove_file(socket_path)?;
                }
                Some(server.bind_uds(socket_path)?)
            } else {
                let bind_address = format!(
                    "{}:{}",
                    server_config.server_host, server_config.server_port
                );
                warn!("Starting server at http://{}", bind_address);
                Some(server.bind(bind_address)?)
            }
        }

        #[cfg(not(unix))]
        {
            let bind_address = format!(
                "{}:{}",
                server_config.server_host, server_config.server_port
            );
            warn!("Starting server at http://{}", bind_address);
            Some(server.bind(bind_address)?)
        }
    }
    .expect("Server binding failed")
    .run();

    // Wait for server or shutdown signal
    tokio::select! {
        res = server => {
            res?;
        }
        _ = lifetime::shutdown::listen_for_shutdown(&db_for_shutdown) => {
            warn!("Graceful shutdown: all tasks completed");
        }
    }

    Ok(())
}
