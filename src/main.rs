use actix_web::{get, App, HttpResponse, HttpServer, Responder, web};
use dotenv::dotenv;
use std::env;
use log::{info, error};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fs;
use serde::{Deserialize, Serialize};
use std::thread;
use std::process;
use std::time::{Duration, SystemTime};

// 条件编译：仅在Unix系统导入信号相关
#[cfg(unix)]
use signal_hook::{consts::SIGUSR1, iterator::Signals};
#[cfg(unix)]
use nix::sys::signal::{self, Signal};
#[cfg(unix)]
use nix::unistd::Pid;

// 配置结构体
#[derive(Clone, Debug)]
struct Config {
    server_host: String,
    server_port: u16,
    links_file: String,
}

// 短链接数据结构
#[derive(Serialize, Deserialize, Clone, Debug)]
struct ShortLink {
    short_code: String,
    target_url: String,
}

type LinkStorage = Arc<RwLock<HashMap<String, String>>>;

// 从文件加载短链接
fn load_links(file_path: &str) -> HashMap<String, String> {
    match fs::read_to_string(file_path) {
        Ok(content) => {
            match serde_json::from_str::<Vec<ShortLink>>(&content) {
                Ok(links) => {
                    let mut map = HashMap::new();
                    for link in links {
                        map.insert(link.short_code, link.target_url);
                    }
                    info!("加载了 {} 个短链接", map.len());
                    map
                }
                Err(e) => {
                    error!("解析链接文件失败: {}", e);
                    HashMap::new()
                }
            }
        }
        Err(_) => {
            info!("链接文件不存在，创建空的存储");
            HashMap::new()
        }
    }
}

// 保存短链接到文件
fn save_links(links: &HashMap<String, String>, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let links_vec: Vec<ShortLink> = links.iter()
        .map(|(short_code, target_url)| ShortLink {
            short_code: short_code.clone(),
            target_url: target_url.clone(),
        })
        .collect();
    
    let json = serde_json::to_string_pretty(&links_vec)?;
    fs::write(file_path, json)?;
    Ok(())
}

#[cfg(unix)]
fn notify_server() -> Result<(), Box<dyn std::error::Error>> {
    // Read the PID from file and send SIGUSR1 to the server process
    match fs::read_to_string("shortlinker.pid") {
        Ok(pid_str) => {
            let pid: i32 = pid_str.trim().parse()?;
            signal::kill(Pid::from_raw(pid), Signal::SIGUSR1)?;
            println!("已通知服务器重新加载配置");
            Ok(())
        }
        Err(_) => {
            println!("警告: 无法找到服务器进程，请手动重启服务器");
            Ok(())
        }
    }
}

#[cfg(windows)]
fn notify_server() -> Result<(), Box<dyn std::error::Error>> {
    // Windows平台使用触发文件方式
    match fs::write("shortlinker.reload", "") {
        Ok(_) => {
            println!("已通知服务器重新加载配置");
            Ok(())
        }
        Err(e) => {
            println!("通知服务器失败: {}", e);
            Err(Box::new(e))
        }
    }
}

// Unix平台的信号监听
#[cfg(unix)]
fn setup_reload_mechanism(links: Arc<RwLock<HashMap<String, String>>>, links_file: String) {
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGUSR1]).unwrap();
        for _ in signals.forever() {
            info!("收到 SIGUSR1 信号，重新加载链接配置");
            let new_links = load_links(&links_file);
            let mut links_map = links.write().unwrap();
            *links_map = new_links;
        }
    });
}

// Windows平台的文件监听
#[cfg(windows)]
fn setup_reload_mechanism(links: Arc<RwLock<HashMap<String, String>>>, links_file: String) {
    thread::spawn(move || {
        let reload_file = "shortlinker.reload";
        let mut last_check = SystemTime::now();
        
        loop {
            thread::sleep(Duration::from_millis(3000));
            
            if let Ok(metadata) = fs::metadata(reload_file) {
                if let Ok(modified) = metadata.modified() {
                    if modified > last_check {
                        info!("检测到重新加载请求，重新加载链接配置");
                        let new_links = load_links(&links_file);
                        let mut links_map = links.write().unwrap();
                        *links_map = new_links;
                        last_check = SystemTime::now();
                        
                        // 删除触发文件
                        let _ = fs::remove_file(reload_file);
                    }
                }
            }
        }
    });
}

// CLI Mode
fn run_cli() {
    let args: Vec<String> = env::args().collect();
    let links_file = env::var("LINKS_FILE").unwrap_or_else(|_| "links.json".to_string());
    let mut links = load_links(&links_file);
    
    match args[1].as_str() {
        "add" => {
            if args.len() != 4 {
                println!("用法: {} add <短码> <目标URL>", args[0]);
                process::exit(1);
            }
            
            let short_code = &args[2];
            let target_url = &args[3];
            
            // 验证 URL 格式
            if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
                println!("错误: URL 必须以 http:// 或 https:// 开头");
                process::exit(1);
            }
            
            links.insert(short_code.clone(), target_url.clone());
            
            if let Err(e) = save_links(&links, &links_file) {
                println!("保存失败: {}", e);
                process::exit(1);
            }
            
            println!("已添加短链接: {} -> {}", short_code, target_url);
            let _ = notify_server();
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
                let _ = notify_server();
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
async fn shortlinker(tail: web::Path<String>, links: web::Data<LinkStorage>) -> impl Responder {
    let captured_path = tail.to_string();

    if captured_path == "" {
        info!("跳转到主页");
        return HttpResponse::TemporaryRedirect()
            .append_header(("Location", "https://www.esaps.net/"))
            .finish();
    } else {
        // Find the target URL in the links map
        let links_map = links.read().unwrap();
        if let Some(target_url) = links_map.get(&captured_path) {
            info!("重定向 {} -> {}", captured_path, target_url);
            return HttpResponse::TemporaryRedirect()
                .append_header(("Location", target_url.as_str()))
                .finish();
        } else {
            return HttpResponse::Ok()
                .content_type("text/plain")
                .body(format!("短链接不存在: {}", captured_path));
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
    setup_reload_mechanism(links.clone(), config.links_file.clone());
    
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