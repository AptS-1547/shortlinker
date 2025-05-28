use std::env;
use std::process;
use crate::utils::generate_random_code;
use crate::storages::{STORAGE, ShortLink};
use crate::signal;

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
                println!("用法: {} add <短码> <目标URL> [--force] [--expire <时间>]", args[0]);
                println!("或使用随机短码: {} add <目标URL> [--force] [--expire <时间>]", args[0]);
                println!("  --force: 强制覆盖已存在的短码");
                println!("  --expire: 设置过期时间");
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
                            println!("错误: --expire 需要指定时间参数");
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
                (random_code, positional_args[0].clone())
            } else if positional_args.len() == 2 {
                // 使用指定短码
                (positional_args[0].clone(), positional_args[1].clone())
            } else {
                println!("错误: 参数数量不正确");
                println!("用法: {} add <短码> <目标URL> [--force] [--expire <时间>]", args[0]);
                println!("或使用随机短码: {} add <目标URL> [--force] [--expire <时间>]", args[0]);
                process::exit(1);
            };
            
            // 验证 URL 格式
            if !target_url.starts_with("http://") && !target_url.starts_with("https://") {
                println!("错误: URL 必须以 http:// 或 https:// 开头");
                process::exit(1);
            }
            
            // 检查短码是否已存在
            if links.contains_key(&short_code) {
                if force_overwrite {
                    println!("强制覆盖短码 '{}': {} -> {}", short_code, links[&short_code].target, target_url);
                } else {
                    println!("错误: 短码 '{}' 已存在，当前指向: {}", short_code, links[&short_code].target);
                    println!("如需覆盖，请使用 --force 参数");
                    process::exit(1);
                }
            }

            let expire_time = if let Some(expire) = expire_time {
                match chrono::DateTime::parse_from_rfc3339(&expire) {
                    Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
                    Err(_) => {
                        println!("错误: 过期时间格式不正确，应为 RFC3339 格式");
                        println!("示例: 2023-10-01T12:00:00Z");
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
                expires_at: expire_time.clone(),
            };
            
            if let Err(e) = STORAGE.set(link).await {
                println!("保存失败: {}", e);
                process::exit(1);
            }
            
            if let Some(expire) = expire_time {
                println!("已添加短链接: {} -> {} (过期时间: {})", short_code, target_url, expire);
            } else {
                println!("已添加短链接: {} -> {}", short_code, target_url);
            }
            let _ = signal::notify_server();
        }
        
        "remove" => {
            if args.len() != 3 {
                println!("用法: {} remove <短码>", args[0]);
                process::exit(1);
            }
            
            let short_code = &args[2];
            
            if links.contains_key(short_code) {
                STORAGE.remove(short_code).await.unwrap_or_else(|e| {
                    println!("删除短链接失败: {}", e);
                    process::exit(1);
                });
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
                for (short_code, link) in &links {
                    if let Some(expires_at) = link.expires_at {
                        println!("  {} -> {} (过期: {})", short_code, link.target, expires_at.format("%Y-%m-%d %H:%M:%S UTC"));
                    } else {
                        println!("  {} -> {}", short_code, link.target);
                    }
                }
            }
        }

        _ => {
            println!("CLI 用法:");
            println!("  {} add <短码> <目标URL> [--force] [--expire <时间>]", args[0]);
            println!("  {} add <目标URL> [--force] [--expire <时间>]  # 使用随机短码", args[0]);
            println!("  {} remove <短码>", args[0]);
            println!("  {} list", args[0]);
            process::exit(1);
        }
    }
}
