use actix_web::{get, App, HttpResponse, HttpServer, Responder, web};
use dotenv::dotenv;
use std::env;
use log::{info, error};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fs;
use std::process;

mod reload;
mod signal;
mod storages;
mod utils;
mod cli;

use storages::STORAGE;

// 配置结构体
#[derive(Clone, Debug)]
struct Config {
    server_host: String,
    server_port: u16,
}

type LinkStorage = Arc<RwLock<HashMap<String, storages::ShortLink>>>;

#[get("/{path}*")]
async fn shortlinker(path: web::Path<String>, links: web::Data<LinkStorage>) -> impl Responder {
    let captured_path = path.to_string();

    if captured_path == "" {
        let default_url = env::var("DEFAULT_URL").unwrap_or_else(|_| "https://esap.cc/repo".to_string());
        info!("重定向到默认主页: {}", default_url);
        return HttpResponse::TemporaryRedirect()
            .append_header(("Location", default_url))
            .finish();
    } else {
        // Find the target URL in the links map
        let links_map = links.read().unwrap();
        if let Some(link) = links_map.get(&captured_path) {
            // Check if the link has expired
            if let Some(expires_at) = link.expires_at {
                if expires_at < chrono::Utc::now() {
                    info!("链接已过期: {}", captured_path);
                    return HttpResponse::NotFound()
                        .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                        .content_type("text/plain")
                        .body("Not Found");
                }
            }

            info!("重定向 {} -> {}", captured_path, link.target);
            return HttpResponse::TemporaryRedirect()
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .append_header(("Location", link.target.as_str()))
                .finish();
        } else {
            return HttpResponse::NotFound()
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .content_type("text/plain")
                .body("Not Found");
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // CLI Mode
    if args.len() > 1 {
        cli::run_cli().await;
        return Ok(());
    }

    // Server Mode
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Load env configurations
    let config = Config {
        server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        server_port: env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap(),
    };
    
    // Save Server PID to file (仅Unix系统需要)
    #[cfg(unix)]
    {
        use std::path::Path;
        use nix::sys::signal;
        use nix::unistd::Pid;
        
        let pid_file = "shortlinker.pid";
        
        // 检查是否已有 PID 文件存在
        if Path::new(pid_file).exists() {
            match fs::read_to_string(pid_file) {
                Ok(old_pid_str) => {
                    if let Ok(old_pid) = old_pid_str.trim().parse::<u32>() {
                        // 检查该 PID 的进程是否仍在运行
                        if signal::kill(Pid::from_raw(old_pid as i32), None).is_ok() {
                            error!("服务器已在运行 (PID: {})，请先停止现有进程", old_pid);
                            error!("可以使用以下命令停止:");
                            error!("  kill {}", old_pid);
                            process::exit(1);
                        } else {
                            // 进程已停止，清理旧的 PID 文件
                            info!("检测到孤立的 PID 文件，清理中...");
                            let _ = fs::remove_file(pid_file);
                        }
                    }
                }
                Err(_) => {
                    // PID 文件损坏，删除它
                    let _ = fs::remove_file(pid_file);
                }
            }
        }
        
        // 写入当前进程的 PID
        let pid = process::id();
        if let Err(e) = fs::write(pid_file, pid.to_string()) {
            error!("无法写入PID文件: {}", e);
        } else {
            info!("服务器 PID: {}", pid);
        }
    }

    // Load links from file
    let links_data = STORAGE.load_all().await;
    let cache = Arc::new(RwLock::new(links_data));
    
    // 设置重新加载机制（根据平台不同）
    reload::setup_reload_mechanism(cache.clone());
    
    let bind_address = format!("{}:{}", config.server_host, config.server_port);
    info!("Starting server at http://{}", bind_address);
    
    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(cache.clone()))
            .service(shortlinker)
    })
    .bind(bind_address)?
    .run()
    .await
}
// DONE