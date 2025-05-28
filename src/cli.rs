use std::env;
use std::process;
use crate::utils::generate_random_code;
use crate::storages::{STORAGE, ShortLink};
use crate::signal;

// ANSI 颜色码
const RESET: &str = "\x1b[0m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

// 彩色输出宏
macro_rules! print_error {
    ($($arg:tt)*) => {
        println!("{}{}错误:{} {}", BOLD, RED, RESET, format!($($arg)*));
    };
}

macro_rules! print_success {
    ($($arg:tt)*) => {
        println!("{}{}✓{} {}", BOLD, GREEN, RESET, format!($($arg)*));
    };
}

macro_rules! print_info {
    ($($arg:tt)*) => {
        println!("{}{}ℹ{} {}", BOLD, BLUE, RESET, format!($($arg)*));
    };
}

macro_rules! print_warning {
    ($($arg:tt)*) => {
        println!("{}{}⚠{} {}", BOLD, YELLOW, RESET, format!($($arg)*));
    };
}

macro_rules! print_usage {
    ($($arg:tt)*) => {
        println!("{}{}{}", CYAN, format!($($arg)*), RESET);
    };
}

pub async fn run_cli() {
    let args: Vec<String> = env::args().collect();
    let random_code_length: usize = env::var("RANDOM_CODE_LENGTH")
        .unwrap_or_else(|_| "6".to_string())
        .parse()
        .unwrap_or(6);
    let links = STORAGE.load_all().await;
    
    match args[1].as_str() {
        "add" => {
            if args.len() < 3 {
                print_usage!("用法:");
                print_usage!("  {} add <短码> <目标URL> [--force] [--expire <时间>]", args[0]);
                print_usage!("  {} add <目标URL> [--force] [--expire <时间>]  # 使用随机短码", args[0]);
                println!("{}选项:{}", BOLD, RESET);
                println!("  {}--force{}   强制覆盖已存在的短码", YELLOW, RESET);
                println!("  {}--expire{}  设置过期时间", YELLOW, RESET);
                process::exit(1);
            }

            let mut force_overwrite = false;
            let mut expire_time: Option<String> = None;
            let mut positional_args = Vec::new();
            
            // 使用for循环解析参数
            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--force" => {
                        force_overwrite = true;
                        i += 1;
                    }
                    "--expire" => {
                        if i + 1 < args.len() {
                            expire_time = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            print_error!("--expire 需要指定时间参数");
                            process::exit(1);
                        }
                    }
                    _ => {
                        positional_args.push(args[i].clone());
                        i += 1;
                    }
                }
            }
            
            let (short_code, target_url) = if positional_args.len() == 1 {
                // 使用随机短码
                let random_code = generate_random_code(random_code_length);
                print_info!("生成随机短码: {}{}{}", MAGENTA, random_code, RESET);
                (random_code, positional_args[0].clone())
            } else if positional_args.len() == 2 {
                // 使用指定短码
                (positional_args[0].clone(), positional_args[1].clone())
            } else {
                print_error!("参数数量不正确");
                print_usage!("用法:");
                print_usage!("  {} add <短码> <目标URL> [--force] [--expire <时间>]", args[0]);
                print_usage!("  {} add <目标URL> [--force] [--expire <时间>]", args[0]);
                process::exit(1);
            };
            
            // 验证 URL 格式
            if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
                print_error!("URL 必须以 http:// 或 https:// 开头");
                process::exit(1);
            }
            
            // 检查短码是否已存在
            if links.contains_key(&short_code) {
                if force_overwrite {
                    print_warning!("强制覆盖短码 '{}{}{}': {}{}{} -> {}{}{}", 
                        CYAN, short_code, RESET,
                        DIM, links[&short_code].target, RESET,
                        BLUE, target_url, RESET
                    );
                } else {
                    print_error!("短码 '{}{}{}' 已存在，当前指向: {}{}{}", 
                        CYAN, short_code, RESET,
                        BLUE, links[&short_code].target, RESET
                    );
                    println!("{}如需覆盖，请使用 {}--force{} 参数{}", DIM, YELLOW, DIM, RESET);
                    process::exit(1);
                }
            }

            let expire_time = if let Some(expire) = expire_time {
                match chrono::DateTime::parse_from_rfc3339(&expire) {
                    Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
                    Err(_) => {
                        print_error!("过期时间格式不正确，应为 RFC3339 格式，如 2023-10-01T12:00:00Z");
                        process::exit(1);
                    }
                }
            } else {
                None
            };
            
            // 创建ShortLink结构
            let link = ShortLink {
                code: short_code.clone(),
                target: target_url.clone(),
                created_at: chrono::Utc::now(),
                expires_at: expire_time,
            };
            
            if let Err(e) = STORAGE.set(link).await {
                print_error!("保存失败: {}", e);
                process::exit(1);
            }
            
            if let Some(expire) = expire_time {
                print_success!("已添加短链接: {}{}{} -> {}{}{} (过期时间: {}{}{})", 
                    CYAN, short_code, RESET,
                    BLUE, target_url, RESET,
                    YELLOW, expire.format("%Y-%m-%d %H:%M:%S UTC"), RESET
                );
            } else {
                print_success!("已添加短链接: {}{}{} -> {}{}{}", 
                    CYAN, short_code, RESET,
                    BLUE, target_url, RESET
                );
            }
            let _ = signal::notify_server();
        }
        
        "remove" => {
            if args.len() != 3 {
                print_usage!("用法: {} remove <短码>", args[0]);
                process::exit(1);
            }
            
            let short_code = &args[2];
            
            if links.contains_key(short_code) {
                match STORAGE.remove(short_code).await {
                    Ok(_) => {
                        print_success!("已删除短链接: {}{}{}", CYAN, short_code, RESET);
                        let _ = signal::notify_server();
                    }
                    Err(e) => {
                        print_error!("删除失败: {}", e);
                        process::exit(1);
                    }
                }
            } else {
                print_error!("短链接不存在: {}{}{}", CYAN, short_code, RESET);
                process::exit(1);
            }
        }

        "list" => {
            if links.is_empty() {
                print_info!("没有短链接");
            } else {
                println!("{}{}短链接列表:{}", BOLD, GREEN, RESET);
                println!();
                for (short_code, link) in &links {
                    if let Some(expires_at) = link.expires_at {
                        println!("  {}{}{} -> {}{}{} {}(过期: {}{}{}){}", 
                            CYAN, short_code, RESET,
                            BLUE, link.target, RESET,
                            DIM, YELLOW, expires_at.format("%Y-%m-%d %H:%M:%S UTC"), DIM, RESET
                        );
                    } else {
                        println!("  {}{}{} -> {}{}{}", 
                            CYAN, short_code, RESET,
                            BLUE, link.target, RESET
                        );
                    }
                }
                println!();
                print_info!("共 {}{}{} 个短链接", GREEN, links.len(), RESET);
            }
        }

        _ => {
            println!("{}{}短链接管理工具{}", BOLD, MAGENTA, RESET);
            println!();
            print_usage!("CLI 用法:");
            print_usage!("  {} add <短码> <目标URL> [--force] [--expire <时间>]", args[0]);
            print_usage!("  {} add <目标URL> [--force] [--expire <时间>]  # 使用随机短码", args[0]);
            print_usage!("  {} remove <短码>", args[0]);
            print_usage!("  {} list", args[0]);
            println!();
            println!("{}选项:{}", BOLD, RESET);
            println!("  {}--force{}   强制覆盖已存在的短码", YELLOW, RESET);
            println!("  {}--expire{}  设置过期时间", YELLOW, RESET);
            process::exit(1);
        }
    }
}
