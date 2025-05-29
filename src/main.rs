use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use log::{debug, error, info};
use std::env;
use std::fs;

#[cfg(unix)]
use std::process;

mod cli;
mod reload;
mod signal;
mod storages;
mod utils;

mod admin;

use storages::STORAGE;

// 配置结构体
#[derive(Clone, Debug)]
struct Config {
    server_host: String,
    server_port: u16,
}

#[actix_web::route("/{path}*", method = "GET", method = "HEAD")]
async fn shortlinker(path: web::Path<String>) -> impl Responder {
    let captured_path = path.to_string();

    debug!("捕获的路径: {}", captured_path);

    if captured_path.is_empty() {
        let default_url =
            env::var("DEFAULT_URL").unwrap_or_else(|_| "https://esap.cc/repo".to_string());
        info!("重定向到默认主页: {}", default_url);
        HttpResponse::TemporaryRedirect()
            .append_header(("Location", default_url))
            .finish()
    } else {
        // 使用 STORAGE 获取链接
        if let Some(link) = STORAGE.get(&captured_path).await {
            // 检查链接是否过期
            if let Some(expires_at) = link.expires_at {
                if expires_at < chrono::Utc::now() {
                    info!("链接已过期: {}", captured_path);
                    return HttpResponse::NotFound()
                        .append_header(("Content-Type", "text/html; charset=utf-8"))
                        .append_header(("Connection", "close"))
                        .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                        .body("Not Found");
                }
            }

            info!("重定向 {} -> {}", captured_path, link.target);
            HttpResponse::TemporaryRedirect()
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .append_header(("Location", link.target.as_str()))
                .finish()
        } else {
            HttpResponse::NotFound()
                .append_header(("Content-Type", "text/html; charset=utf-8"))
                .append_header(("Connection", "close"))
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .body("Not Found")
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
        server_port: env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap(),
    };

    // Save Server PID to file (仅Unix系统需要)
    #[cfg(unix)]
    {
        use nix::sys::signal;
        use nix::unistd::Pid;
        use std::path::Path;

        let pid_file = "shortlinker.pid";

        // 检查是否已有 PID 文件存在
        if Path::new(pid_file).exists() {
            match fs::read_to_string(pid_file) {
                Ok(old_pid_str) => {
                    if let Ok(old_pid) = old_pid_str.trim().parse::<u32>() {
                        // 在 Docker 容器中，如果是 PID 1 且文件存在，可能是重启后的残留
                        let current_pid = process::id();

                        // 如果当前进程是 PID 1 且旧 PID 也是 1，说明是容器重启
                        if current_pid == 1 && old_pid == 1 {
                            info!("检测到容器重启，清理旧的 PID 文件");
                            let _ = fs::remove_file(pid_file);
                        } else if signal::kill(Pid::from_raw(old_pid as i32), None).is_ok() {
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

    // Windows 系统写入 .shortlinker.lock 防止重复启动
    #[cfg(windows)]
    {
        use std::io::{self, Write};
        use std::path::Path;

        let lock_file = ".shortlinker.lock";
        // 检查是否已有锁文件存在
        if Path::new(lock_file).exists() {
            error!("服务器已在运行，请先停止现有进程");
            error!("如果确认服务器没有运行，可以删除锁文件: {}", lock_file);
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "Server is already running",
            ));
        }

        match fs::File::create(lock_file) {
            Ok(mut file) => {
                if let Err(e) = writeln!(file, "Server is running") {
                    error!("无法写入锁文件: {}", e);
                }
            }
            Err(e) => {
                error!("无法创建锁文件: {}", e);
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Failed to create lock file",
                ));
            }
        }
    }

    // 设置重载机制
    debug!("Setting up reload mechanism");
    reload::setup_reload_mechanism();

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
            // 使用 scope 设置 admin 路由前缀
            .service(
                web::scope(&admin_prefix_clone)
                    .service(admin::get_all_links)
                    .service(admin::post_link)
                    .service(admin::get_link)
                    .service(admin::delete_link)
                    .service(admin::update_link),
            )
            .service(shortlinker)
    })
    .bind(bind_address)?
    .run()
    .await?;

    // Clean up PID file on exit
    #[cfg(unix)]
    {
        let pid_file = "shortlinker.pid";
        if let Err(e) = fs::remove_file(pid_file) {
            error!("无法删除PID文件: {}", e);
        } else {
            info!("已清理PID文件: {}", pid_file);
        }
    }
    #[cfg(windows)]
    {
        let lock_file = ".shortlinker.lock";
        if let Err(e) = fs::remove_file(lock_file) {
            error!("无法删除锁文件: {}", e);
        } else {
            info!("已清理锁文件: {}", lock_file);
        }
    }

    Ok(())
}
// DONE
