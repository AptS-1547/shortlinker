use actix_web::{
    middleware::{from_fn, DefaultHeaders},
    web, App, HttpServer,
};
use dotenv::dotenv;
use std::env;
use tracing::{debug, warn};

mod cli;
mod errors;
mod middleware;
mod services;
mod storages;
mod system;
mod utils;

use crate::middleware::{AuthMiddleware, HealthMiddleware};
use crate::services::{AdminService, AppStartTime, HealthService, RedirectService};
use crate::storages::StorageFactory;
use crate::system::{cleanup_lockfile, init_lockfile};

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
    // 记录程序启动时间
    let app_start_time = AppStartTime {
        start_datetime: chrono::Utc::now(),
    };

    let args: Vec<String> = env::args().collect();
    dotenv().ok();

    // CLI Mode
    if args.len() > 1 {
        env::set_var("CLI_MODE", "true");
        cli::run_cli().await;
        return Ok(());
    }

    // Server Mode
    env::set_var("CLI_MODE", "false");

    // Initialize tracing subscriber
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

    // 初始化锁文件
    init_lockfile()?;

    // 检查存储后端
    let storage = StorageFactory::create()
        .await
        .expect("Failed to create storage backend");
    warn!(
        "Using storage backend: {}",
        storage.get_backend_name().await
    );

    // 设置重载机制
    debug!("Setting up reload mechanism");
    system::setup_reload_mechanism(storage.clone());

    // 获取路由前缀配置
    let admin_prefix = env::var("ADMIN_ROUTE_PREFIX").unwrap_or_else(|_| "/admin".to_string());
    let health_prefix = env::var("HEALTH_ROUTE_PREFIX").unwrap_or_else(|_| "/health".to_string());

    // 检查 Admin API 是否启用
    let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| "".to_string());
    if admin_token.is_empty() {
        warn!("Admin API is disabled (ADMIN_TOKEN not set)");
    } else {
        warn!("Admin API available at: {}", admin_prefix);
    }

    // 检查 Health API 是否启用
    let health_token = env::var("HEALTH_TOKEN").unwrap_or_default();
    if health_token.is_empty() {
        warn!("Health API is disabled (HEALTH_TOKEN is empty)");
    } else {
        warn!("Health API available at: {}", health_prefix);
    }

    let cpu_count = env::var("CPU_COUNT")
        .unwrap_or_else(|_| num_cpus::get().to_string())
        .parse::<usize>()
        .unwrap_or_else(|_| num_cpus::get())
        .min(32);

    warn!("Using {} CPU cores for the server", cpu_count);

    // Start the HTTP server
    let server = HttpServer::new(move || {
        App::new()
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
                    .wrap(from_fn(AuthMiddleware::admin_auth))
                    .route("/link", web::get().to(AdminService::get_all_links))
                    .route("/link", web::head().to(AdminService::get_all_links))
                    .route("/link", web::post().to(AdminService::post_link))
                    .route("/link/{code}", web::get().to(AdminService::get_link))
                    .route("/link/{code}", web::head().to(AdminService::get_link))
                    .route("/link/{code}", web::delete().to(AdminService::delete_link))
                    .route("/link/{code}", web::put().to(AdminService::update_link)),
            )
            .service(
                web::scope(&health_prefix)
                    .wrap(from_fn(HealthMiddleware::health_auth))
                    .route("", web::get().to(HealthService::health_check))
                    .route("", web::head().to(HealthService::health_check))
                    .route("/ready", web::get().to(HealthService::readiness_check))
                    .route("/ready", web::head().to(HealthService::readiness_check))
                    .route("/live", web::get().to(HealthService::liveness_check))
                    .route("/live", web::head().to(HealthService::liveness_check)),
            )
            .route("/{path}*", web::get().to(RedirectService::handle_redirect))
            .route("/{path}*", web::head().to(RedirectService::handle_redirect))
    })
    .keep_alive(std::time::Duration::from_secs(30)) // 启用长连接
    .client_request_timeout(std::time::Duration::from_millis(5000)) // 客户端超时
    .client_disconnect_timeout(std::time::Duration::from_millis(1000)) // 断连超时
    .workers(cpu_count);

    #[cfg(unix)]
    {
        if let Some(ref socket_path) = config.unix_socket_path {
            warn!("Starting server on Unix socket: {}", socket_path);
            // 如果 socket 文件已存在，先删除
            if std::path::Path::new(socket_path).exists() {
                std::fs::remove_file(socket_path)?;
            }
            server.bind_uds(socket_path)?.run().await?;
        } else {
            let bind_address = format!("{}:{}", config.server_host, config.server_port);
            warn!("Starting server at http://{}", bind_address);
            server.bind(bind_address)?.run().await?;
        }
    }

    #[cfg(not(unix))]
    {
        let bind_address = format!("{}:{}", config.server_host, config.server_port);
        warn!("Starting server at http://{}", bind_address);
        server.bind(bind_address)?.run().await?;
    }

    // 清理锁文件
    cleanup_lockfile();

    Ok(())
}
// DONE
