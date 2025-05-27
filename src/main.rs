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
mod storage;
mod utils;

use storage::{load_links, save_links};
use utils::generate_random_code;

// 配置结构体
#[derive(Clone, Debug)]
struct Config {
    server_host: String,
    server_port: u16,
    links_file: String,
}

type LinkStorage = Arc<RwLock<HashMap<String, String>>>;

// CLI Mode
fn run_cli() {
    let args: Vec<String> = env::args().collect();
    let links_file = env::var("LINKS_FILE").unwrap_or_else(|_| "links.json".to_string());
    let random_code_length: usize = env::var("RANDOM_CODE_LENGTH")
        .unwrap_or_else(|_| "6".to_string())
        .parse()
        .unwrap_or(6);
    let mut links = load_links(&links_file);
    
    match args[1].as_str() {
        "add" => {
            if args.len() != 3 && args.len() != 4 && args.len() != 5 {
                println!("用法: {} add <短码> <目标URL> [--force]", args[0]);
                println!("或使用随机短码: {} add <目标URL>", args[0]);
                println!("  --force: 强制覆盖已存在的短码");
                process::exit(1);
            }

            let mut force_overwrite = false;
            let (short_code, target_url) = if args.len() == 3 {
                // 使用随机短码
                let random_code = generate_random_code(random_code_length);
                (random_code, args[2].clone())
            } else if args.len() == 4 && args[3] != "--force" {
                // 使用指定短码
                (args[2].clone(), args[3].clone())
            } else if args.len() == 4 && args[3] == "--force" {
                // 随机短码 + force
                let random_code: String = generate_random_code(random_code_length);
                force_overwrite = true;
                (random_code, args[2].clone())
            } else {
                // 指定短码 + force
                if args[4] == "--force" {
                    force_overwrite = true;
                }
                (args[2].clone(), args[3].clone())
            };
            
            // 验证 URL 格式
            if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
                println!("错误: URL 必须以 http:// 或 https:// 开头");
                process::exit(1);
            }
            
            // 检查短码是否已存在
            if links.contains_key(&short_code) {
                if force_overwrite {
                    println!("强制覆盖短码 '{}': {} -> {}", short_code, links[&short_code], target_url);
                } else {
                    println!("错误: 短码 '{}' 已存在，当前指向: {}", short_code, links[&short_code]);
                    println!("如需覆盖，请使用 --force 参数");
                    process::exit(1);
                }
            }
            
            links.insert(short_code.clone(), target_url.clone());
            
            if let Err(e) = save_links(&links, &links_file) {
                println!("保存失败: {}", e);
                process::exit(1);
            }
            
            println!("已添加短链接: {} -> {}", short_code, target_url);
            let _ = signal::notify_server();
        }
        
        "remove" => {
            if args.len() != 3 {
                println!("用法: {} remove <短码>", args[0]);
                process::exit(1);
            }
            
            let short_code = &args[2];
            
            if links.remove(short_code).is_some() {
                if let Err(e) = save_links(&links, &links_file) {
                    error!("保存失败: {}", e);
                    process::exit(1);
                }
                println!("已删除短链接: {}", short_code);
                let _ = signal::notify_server();
            } else {
                println!("短链接不存在: {}", short_code);
                process::exit(1);
            }
        }

        "list" => {
            if links.is_empty() {
                println!("没有短链接");
            } else {
                println!("短链接列表:");
                for (short_code, target_url) in &links {
                    println!("  {} -> {}", short_code, target_url);
                }
            }
        }

        _ => {
            println!("CLI 用法:");
            println!("  {} add <短码> <目标URL>", args[0]);
            println!("  {} remove <短码>", args[0]);
            println!("  {} list", args[0]);
            process::exit(1);
        }
    }
}

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
        if let Some(target_url) = links_map.get(&captured_path) {
            info!("重定向 {} -> {}", captured_path, target_url);
            return HttpResponse::TemporaryRedirect()
                .append_header(("Cache-Control", "no-cache, no-store, must-revalidate"))
                .append_header(("Location", target_url.as_str()))
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
        run_cli();
        return Ok(());
    }

    // Server Mode
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Load env configurations
    let config = Config {
        server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        server_port: env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap(),
        links_file: env::var("LINKS_FILE").unwrap_or_else(|_| "links.json".to_string()),
    };
    
    // Save Server PID to file (仅Unix系统需要)
    #[cfg(unix)]
    {
        let pid = process::id();
        if let Err(e) = fs::write("shortlinker.pid", pid.to_string()) {
            error!("无法写入PID文件: {}", e);
        }
    }
    
    // Load links from file
    let links = Arc::new(RwLock::new(load_links(&config.links_file)));
    
    // 设置重新加载机制（根据平台不同）
    reload::setup_reload_mechanism(links.clone(), config.links_file.clone());
    
    let bind_address = format!("{}:{}", config.server_host, config.server_port);
    info!("Starting server at http://{}", bind_address);
    
    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(links.clone()))
            .service(shortlinker)
    })
    .bind(bind_address)?
    .run()
    .await
}