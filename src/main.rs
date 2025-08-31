use actix_web::{App, HttpServer, middleware::DefaultHeaders, web};
use color_eyre::Result;
use dotenv::dotenv;
use tracing::{debug, warn};
use tracing_appender::rolling;

#[cfg(any(feature = "cli", feature = "tui"))]
use colored::Colorize;

mod cache;
mod errors;
mod event;
mod middleware;
mod services;
mod storages;
mod system;
mod utils;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "tui")]
mod tui;

use crate::middleware::{AdminAuth, FrontendGuard, HealthAuth};
use crate::services::{
    AdminService, AppStartTime, FrontendService, HealthService, RedirectService,
};
use crate::system::lifetime;

// 配置结构体
#[derive(Clone, Debug)]
struct ServerConfig {
    server_host: String,
    server_port: u16,
    #[cfg(unix)]
    unix_socket_path: Option<String>,
}

#[cfg(any(feature = "cli", feature = "tui"))]
async fn handle_mode_error<T, E: std::fmt::Display>(
    result: Result<T, E>,
    mode_name: &str,
) -> Result<(), color_eyre::Report> {
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{} {}: {}", mode_name, "Error".bold().red(), e);
            Err(color_eyre::eyre::eyre!(e.to_string()))
        }
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    // 设置全局错误处理和日志
    color_eyre::install()?;
    dotenv().ok();

    // 初始化配置系统
    crate::system::app_config::init_config();
    let config = crate::system::app_config::get_config();

    #[cfg(any(feature = "cli", feature = "tui"))]
    let args: Vec<String> = std::env::args().collect();

    // 根据编译features决定运行模式
    #[cfg(feature = "tui")]
    if args.len() > 1 && args[1] == "tui" {
        // TUI模式启动逻辑
        return handle_mode_error(tui::run_tui().await, "TUI").await;
    }

    #[cfg(feature = "cli")]
    if args.len() > 1 {
        lifetime::startup::cli_pre_startup().await;
        return handle_mode_error(cli::run_cli().await, "CLI").await;
    }

    // 服务器模式（默认）
    #[cfg(feature = "server")]
    {
        run_server(config).await?;
    }

    #[cfg(not(feature = "server"))]
    {
        eprintln!("No features enabled. Please compile with --features server, cli, or tui");
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(feature = "server")]
async fn run_server(config: &crate::system::app_config::AppConfig) -> Result<()> {
    // 记录程序启动时间
    let app_start_time = AppStartTime {
        start_datetime: chrono::Utc::now(),
    };

    // 启动前预处理 //
    debug!("Starting pre-startup processing...");

    // 初始化日志
    let writer: Box<dyn std::io::Write + Send + Sync> =
        if let Some(ref log_file) = config.logging.file {
            if !log_file.is_empty() && config.logging.enable_rotation {
                // 使用轮转日志
                let dir = std::path::Path::new(log_file)
                    .parent()
                    .unwrap_or(std::path::Path::new("."));
                let filename = std::path::Path::new(log_file)
                    .file_name()
                    .unwrap_or(std::ffi::OsStr::new("shortlinker.log"));
                let filename_str = filename.to_str().unwrap_or("shortlinker.log");
                let appender = rolling::Builder::new()
                    .rotation(rolling::Rotation::DAILY)
                    .filename_prefix(filename_str.trim_end_matches(".log"))
                    .filename_suffix("log")
                    .max_log_files(config.logging.max_backups as usize)
                    .build(dir)
                    .expect("Failed to create rolling log appender");
                Box::new(appender)
            } else if !log_file.is_empty() {
                // 不轮转，直接写入文件
                let file = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(log_file)
                    .expect("Failed to open log file");
                Box::new(file)
            } else {
                // 文件名为空，输出到控制台
                Box::new(std::io::stdout())
            }
        } else {
            // 输出到控制台
            Box::new(std::io::stdout())
        };

    let (non_blocking_writer, _guard) = tracing_appender::non_blocking(writer);
    let filter = tracing_subscriber::EnvFilter::new(config.logging.level.clone());

    let subscriber_builder = tracing_subscriber::fmt()
        .with_writer(non_blocking_writer)
        .with_env_filter(filter)
        .with_level(true)
        .with_ansi(config.logging.file.as_ref().is_none_or(|f| f.is_empty()));

    if config.logging.format == "json" {
        subscriber_builder.json().init();
    } else {
        subscriber_builder.init();
    }

    let startup = lifetime::startup::prepare_server_startup().await;

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

    // Load server configurations from new config system
    let server_config = ServerConfig {
        server_host: config.server.host.clone(),
        server_port: config.server.port,
        #[cfg(unix)]
        unix_socket_path: config.server.unix_socket.clone(),
    };

    let cpu_count = config.server.cpu_count.min(32);

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
