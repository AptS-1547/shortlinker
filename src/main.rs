use actix_web::{middleware::DefaultHeaders, web, App, HttpServer};
use dotenv::dotenv;
use std::env;
use tracing::{debug, warn};

mod cache;
mod cli;
mod errors;
mod middleware;
mod services;
mod storages;
mod system;
mod utils;

use crate::middleware::{AdminAuth, FrontendGuard, HealthAuth};
use crate::services::{
    AdminService, AppStartTime, FrontendService, HealthService, RedirectService,
};

// 配置结构体
#[derive(Clone, Debug)]
struct Config {
    server_host: String,
    server_port: u16,
    #[cfg(unix)]
    unix_socket_path: Option<String>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let args: Vec<String> = env::args().collect();

    // CLI Mode
    if args.len() > 1 {
        system::startup::cli_pre_startup().await;
        cli::run_cli().await;
        return Ok(());
    }

    // 记录程序启动时间
    let app_start_time = AppStartTime {
        start_datetime: chrono::Utc::now(),
    };

    // 启动前预处理 //

    debug!("Starting pre-startup processing...");
    // 初始化日志
    let stdout_log = std::io::stdout();
    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(stdout_log);
    let filter = tracing_subscriber::EnvFilter::new(
        env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
    );
    tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .with_env_filter(filter)
        .with_level(true)
        .with_ansi(true)
        .init();

    let startup = system::startup::prepare_server_startup().await;

    let cache = startup.cache.clone();
    let storage = startup.storage.clone();
    let route = startup.route_config.clone();

    let admin_prefix = route.admin_prefix;
    let health_prefix = route.health_prefix;
    let frontend_prefix = route.frontend_prefix;

    // 输出预处理时间
    debug!(
        "Pre-startup processing completed in {} ms",
        chrono::Utc::now()
            .signed_duration_since(app_start_time.start_datetime)
            .num_milliseconds()
    );

    // 预处理完成 //

    // Load env configurations
    let config = Config {
        server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        server_port: env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap(),
        #[cfg(unix)]
        unix_socket_path: env::var("UNIX_SOCKET").ok(),
    };

    let cpu_count = env::var("CPU_COUNT")
        .unwrap_or_else(|_| num_cpus::get().to_string())
        .parse::<usize>()
        .unwrap_or_else(|_| num_cpus::get())
        .min(32);

    warn!("Using {} CPU cores for the server", cpu_count);

    // Start the HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cache.clone()))
            .app_data(web::Data::new(storage.clone()))
            .app_data(web::Data::new(app_start_time.clone()))
            .app_data(web::PayloadConfig::new(1024 * 1024)) // 设置最大请求体大小为1MB
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
                    ),
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
                    // SPA 路由回退，处理所有其他路径
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
    .keep_alive(std::time::Duration::from_secs(30)) // 启用长连接
    .client_request_timeout(std::time::Duration::from_millis(5000)) // 客户端超时
    .client_disconnect_timeout(std::time::Duration::from_millis(1000)) // 断连超时
    .workers(cpu_count);

    let server = {
        #[cfg(unix)]
        {
            if let Some(ref socket_path) = config.unix_socket_path {
                warn!("Starting server on Unix socket: {}", socket_path);
                if std::path::Path::new(socket_path).exists() {
                    std::fs::remove_file(socket_path)?;
                }
                Some(server.bind_uds(socket_path)?)
            } else {
                let bind_address = format!("{}:{}", config.server_host, config.server_port);
                warn!("Starting server at http://{}", bind_address);
                Some(server.bind(bind_address)?)
            }
        }

        #[cfg(not(unix))]
        {
            let bind_address = format!("{}:{}", config.server_host, config.server_port);
            warn!("Starting server at http://{}", bind_address);
            Some(server.bind(bind_address)?)
        }
    }
    .expect("Server binding failed")
    .run();

    tokio::select! {
        res = server => {
            res?;
        }
        _ = system::shutdown::listen_for_shutdown() => {
            warn!("Graceful shutdown: all tasks completed");
        }
    }

    Ok(())
}
// DONE
