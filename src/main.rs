use actix_web::{middleware::from_fn, web, App, HttpServer};
use dotenv::dotenv;
use log::{debug, info};
use std::env;

mod cli;
mod errors;
mod services;
mod storages;
mod system;
mod utils;

use crate::services::{AdminService, RedirectService};
use crate::storages::StorageFactory;
use crate::system::{cleanup_lockfile, init_lockfile, setup_reload_mechanism};

// 配置结构体
#[derive(Clone, Debug)]
struct Config {
    server_host: String,
    server_port: u16,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    dotenv().ok();

    // CLI Mode
    if args.len() > 1 {
        cli::run_cli().await;
        return Ok(());
    }

    // Server Mode
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Load env configurations
    let config = Config {
        server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        server_port: env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap(),
    };

    // 初始化锁文件
    init_lockfile()?;

    // 检查存储后端
    let storage = StorageFactory::create().expect("Failed to create storage");
    info!(
        "Using storage backend: {}",
        storage.get_backend_name().await
    );

    // 设置重载机制
    debug!("Setting up reload mechanism");
    setup_reload_mechanism(storage.clone());

    // 获取 admin 路由前缀
    let admin_prefix = env::var("ADMIN_ROUTE_PREFIX").unwrap_or_else(|_| "/admin".to_string());
    let admin_prefix_clone = admin_prefix.clone();

    let bind_address = format!("{}:{}", config.server_host, config.server_port);
    info!("Starting server at http://{}", bind_address);

    // 检查 Admin API 是否启用
    let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| "".to_string());
    if admin_token.is_empty() {
        info!("Admin API is disabled (ADMIN_TOKEN not set)");
    } else {
        info!("Admin API available at: {}", admin_prefix);
    }

    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(storage.clone()))
            .service(
                web::scope(&admin_prefix_clone)
                    .wrap(from_fn(AdminService::auth_middleware))
                    .route("/link", web::get().to(AdminService::get_all_links))
                    .route("/link", web::head().to(AdminService::get_all_links))
                    .route("/link", web::post().to(AdminService::post_link))
                    .route("/link/{code}", web::get().to(AdminService::get_link))
                    .route("/link/{code}", web::head().to(AdminService::get_link))
                    .route("/link/{code}", web::delete().to(AdminService::delete_link))
                    .route("/link/{code}", web::put().to(AdminService::update_link)),
            )
            .route("/{path}*", web::get().to(RedirectService::handle_redirect))
            .route("/{path}*", web::head().to(RedirectService::handle_redirect))
    })
    .bind(bind_address)?
    .run()
    .await?;

    // 清理锁文件
    cleanup_lockfile();

    Ok(())
}
// DONE
