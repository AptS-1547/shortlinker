//! Server mode
//!
//! This module contains the HTTP server startup logic.
//! It configures and starts the HTTP server with all necessary routes.

use actix_web::{App, HttpServer, middleware::DefaultHeaders, web};
use anyhow::Result;
use tracing::{debug, warn};

use crate::api::middleware::{AdminAuth, FrontendGuard, HealthAuth};
use crate::api::services::{
    AdminService, AppStartTime, FrontendService, HealthService, RedirectService,
    admin::{get_all_configs, get_config, get_config_history, reload_config, update_config},
};
use crate::runtime::lifetime;

/// Server configuration
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub server_host: String,
    pub server_port: u16,
    #[cfg(unix)]
    pub unix_socket_path: Option<String>,
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
pub async fn run_server(config: &crate::config::AppConfig) -> Result<()> {
    // Record application start time
    let app_start_time = AppStartTime {
        start_datetime: chrono::Utc::now(),
    };

    debug!("Starting pre-startup processing...");

    // Prepare server startup (cache, storage, routes)
    let startup = lifetime::startup::prepare_server_startup().await;

    let cache = startup.cache.clone();
    let storage = startup.storage.clone();
    let route = startup.route_config.clone();

    let admin_prefix = route.admin_prefix;
    let health_prefix = route.health_prefix;
    let frontend_prefix = route.frontend_prefix;

    debug!(
        "Pre-startup processing completed in {} ms",
        chrono::Utc::now()
            .signed_duration_since(app_start_time.start_datetime)
            .num_milliseconds()
    );

    // Load server configuration
    let server_config = ServerConfig {
        server_host: config.server.host.clone(),
        server_port: config.server.port,
        #[cfg(unix)]
        unix_socket_path: config.server.unix_socket.clone(),
    };

    let cpu_count = config.server.cpu_count.min(32);
    warn!("Using {} CPU cores for the server", cpu_count);

    // Configure HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(storage.clone()))
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
                    .wrap(AdminAuth)
                    .route("/link", web::get().to(AdminService::get_all_links))
                    .route("/link", web::head().to(AdminService::get_all_links))
                    .route("/link", web::post().to(AdminService::post_link))
                    // Batch operations (must be before /link/{code:.*} to avoid matching)
                    .route(
                        "/link/batch",
                        web::post().to(AdminService::batch_create_links),
                    )
                    .route(
                        "/link/batch",
                        web::put().to(AdminService::batch_update_links),
                    )
                    .route(
                        "/link/batch",
                        web::delete().to(AdminService::batch_delete_links),
                    )
                    // Stats endpoint (must be before /link/{code:.*} to avoid matching)
                    .route("/stats", web::get().to(AdminService::get_stats))
                    .route("/stats", web::head().to(AdminService::get_stats))
                    .route("/link/{code:.*}", web::get().to(AdminService::get_link))
                    .route("/link/{code:.*}", web::head().to(AdminService::get_link))
                    .route(
                        "/link/{code:.*}",
                        web::delete().to(AdminService::delete_link),
                    )
                    .route("/link/{code:.*}", web::put().to(AdminService::update_link))
                    .route(
                        "/auth/login",
                        web::post().to(AdminService::check_admin_token),
                    )
                    .route("/auth/refresh", web::post().to(AdminService::refresh_token))
                    .route("/auth/logout", web::post().to(AdminService::logout))
                    .route("/auth/verify", web::get().to(AdminService::verify_token))
                    // Config management endpoints
                    .route("/config", web::get().to(get_all_configs))
                    .route("/config/reload", web::post().to(reload_config))
                    .route(
                        "/config/{key:.*}/history",
                        web::get().to(get_config_history),
                    )
                    .route("/config/{key:.*}", web::get().to(get_config))
                    .route("/config/{key:.*}", web::put().to(update_config)),
            )
            .service(
                web::scope(&health_prefix)
                    .wrap(HealthAuth)
                    .route("", web::get().to(HealthService::health_check))
                    .route("", web::head().to(HealthService::health_check))
                    .route("/ready", web::get().to(HealthService::readiness_check))
                    .route("/ready", web::head().to(HealthService::readiness_check))
                    .route("/live", web::get().to(HealthService::liveness_check))
                    .route("/live", web::head().to(HealthService::liveness_check)),
            )
            .service(
                web::scope(&frontend_prefix)
                    .wrap(FrontendGuard)
                    .route("", web::get().to(FrontendService::handle_index))
                    .route("", web::head().to(FrontendService::handle_index))
                    .route(
                        "/assets/{path:.*}",
                        web::get().to(FrontendService::handle_static),
                    )
                    .route(
                        "/assets/{path:.*}",
                        web::head().to(FrontendService::handle_static),
                    )
                    .route("/admin", web::get().to(FrontendService::handle_admin_panel))
                    .route(
                        "/admin",
                        web::head().to(FrontendService::handle_admin_panel),
                    )
                    .route(
                        "/favicon.ico",
                        web::get().to(FrontendService::handle_favicon),
                    )
                    .route(
                        "/favicon.ico",
                        web::head().to(FrontendService::handle_favicon),
                    )
                    .route(
                        "/{path:.*}",
                        web::get().to(FrontendService::handle_spa_fallback),
                    )
                    .route(
                        "/{path:.*}",
                        web::head().to(FrontendService::handle_spa_fallback),
                    ),
            )
            .route("/{path}*", web::get().to(RedirectService::handle_redirect))
            .route("/{path}*", web::head().to(RedirectService::handle_redirect))
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
        _ = lifetime::shutdown::listen_for_shutdown() => {
            warn!("Graceful shutdown: all tasks completed");
        }
    }

    Ok(())
}
