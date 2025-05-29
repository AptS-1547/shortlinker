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
        println!("{}{}错误:{} {}", BOLD, RED, RESET, format!($($arg)*))
    };
}

macro_rules! print_success {
    ($($arg:tt)*) => {
        println!("{}{}✓{} {}", BOLD, GREEN, RESET, format!($($arg)*))
    };
}

macro_rules! print_info {
    ($($arg:tt)*) => {
        println!("{}{}ℹ{} {}", BOLD, BLUE, RESET, format!($($arg)*))
    };
}

macro_rules! print_warning {
    ($($arg:tt)*) => {
        println!("{}{}⚠{} {}", BOLD, YELLOW, RESET, format!($($arg)*))
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
        "help" | "--help" | "-h" => {
            show_help(&args[0]);
        }

        "start" => {
            start_server();
        }

        "stop" => {
            stop_server();
        }

        "restart" => {
            restart_server();
        }

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
            print_error!("未知命令: {}", args[1]);
            println!();
            show_help(&args[0]);
            process::exit(1);
        }
    }
}

fn show_help(program_name: &str) {
    println!("{}{}shortlinker - 短链接管理工具{}", BOLD, MAGENTA, RESET);
    println!();
    println!("{}用法:{}", BOLD, RESET);
    print_usage!("  {}                          # 启动服务器", program_name);
    print_usage!("  {} start                    # 启动服务器", program_name);
    print_usage!("  {} stop                     # 停止服务器", program_name);
    print_usage!("  {} restart                  # 重启服务器", program_name);
    print_usage!("  {} help                     # 显示帮助信息", program_name);
    println!();
    println!("{}链接管理:{}", BOLD, RESET);
    print_usage!("  {} add <短码> <目标URL> [选项]   # 添加短链接", program_name);
    print_usage!("  {} add <目标URL> [选项]         # 使用随机短码添加", program_name);
    print_usage!("  {} remove <短码>              # 删除短链接", program_name);
    print_usage!("  {} list                      # 列出所有短链接", program_name);
    println!();
    println!("{}选项:{}", BOLD, RESET);
    println!("  {}--force{}     强制覆盖已存在的短码", YELLOW, RESET);
    println!("  {}--expires{}   设置过期时间 (RFC3339格式)", YELLOW, RESET);
    println!();
    println!("{}示例:{}", BOLD, RESET);
    println!("  {}启动服务器:{}", DIM, RESET);
    print_usage!("    {} start", program_name);
    println!();
    println!("  {}添加短链接:{}", DIM, RESET);
    print_usage!("    {} add github https://github.com", program_name);
    print_usage!("    {} add https://example.com --expires \"2024-12-31T23:59:59Z\"", program_name);
    println!();
    println!("  {}重启服务器:{}", DIM, RESET);
    print_usage!("    {} restart", program_name);
    println!();
    println!("  {}停止服务器:{}", DIM, RESET);
    print_usage!("    {} stop", program_name);
}

fn start_server() {
    print_info!("正在启动 shortlinker 服务器...");
    
    #[cfg(unix)]
    {
        use std::fs;
        use std::path::Path;
        
        let pid_file = "shortlinker.pid";
        
        // 检查是否已有服务器在运行
        if Path::new(pid_file).exists() {
            match fs::read_to_string(pid_file) {
                Ok(old_pid_str) => {
                    if let Ok(old_pid) = old_pid_str.trim().parse::<u32>() {
                        use nix::sys::signal;
                        use nix::unistd::Pid;
                        
                        // 检查该 PID 的进程是否仍在运行
                        if signal::kill(Pid::from_raw(old_pid as i32), None).is_ok() {
                            print_warning!("服务器已在运行 (PID: {})", old_pid);
                            print_info!("如需重启，请先使用 'stop' 命令停止服务器");
                            return;
                        } else {
                            // 进程已停止，清理旧的 PID 文件
                            print_info!("清理孤立的 PID 文件...");
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
        
        print_info!("使用以下命令启动服务器:");
        print_usage!("  ./shortlinker");
        print_info!("或在后台运行:");
        print_usage!("  nohup ./shortlinker > shortlinker.log 2>&1 &");
    }
    
    #[cfg(windows)]
    {
        use std::path::Path;
        
        let lock_file = ".shortlinker.lock";
        
        if Path::new(lock_file).exists() {
            print_warning!("服务器可能已在运行");
            print_info!("如确认服务器未运行，请删除锁文件: {}", lock_file);
            print_info!("然后重新启动服务器");
            return;
        }
        
        print_info!("使用以下命令启动服务器:");
        print_usage!("  shortlinker.exe");
    }
}

fn stop_server() {
    print_info!("正在停止 shortlinker 服务器...");
    
    #[cfg(unix)]
    {
        use std::fs;
        use std::path::Path;
        
        let pid_file = "shortlinker.pid";
        
        if !Path::new(pid_file).exists() {
            print_warning!("未找到 PID 文件，服务器可能未运行");
            return;
        }
        
        match fs::read_to_string(pid_file) {
            Ok(pid_str) => {
                if let Ok(pid) = pid_str.trim().parse::<u32>() {
                    use nix::sys::signal::{self, Signal};
                    use nix::unistd::Pid;
                    
                    let server_pid = Pid::from_raw(pid as i32);
                    
                    // 首先检查进程是否存在
                    if signal::kill(server_pid, None).is_err() {
                        print_warning!("进程 {} 不存在，清理 PID 文件", pid);
                        let _ = fs::remove_file(pid_file);
                        return;
                    }
                    
                    // 发送 SIGTERM 信号
                    match signal::kill(server_pid, Signal::SIGTERM) {
                        Ok(_) => {
                            print_success!("已向服务器进程 {} 发送停止信号", pid);
                            
                            // 等待一段时间检查进程是否结束
                            std::thread::sleep(std::time::Duration::from_secs(2));
                            
                            // 再次检查进程是否还在运行
                            if signal::kill(server_pid, None).is_ok() {
                                print_warning!("服务器进程仍在运行，尝试强制终止...");
                                match signal::kill(server_pid, Signal::SIGKILL) {
                                    Ok(_) => print_success!("服务器已强制停止"),
                                    Err(e) => print_error!("无法强制停止服务器: {}", e),
                                }
                            } else {
                                print_success!("服务器已正常停止");
                            }
                            
                            // 清理 PID 文件
                            let _ = fs::remove_file(pid_file);
                        }
                        Err(e) => {
                            print_error!("无法停止服务器进程: {}", e);
                        }
                    }
                } else {
                    print_error!("PID 文件格式无效");
                    let _ = fs::remove_file(pid_file);
                }
            }
            Err(e) => {
                print_error!("无法读取 PID 文件: {}", e);
            }
        }
    }
    
    #[cfg(windows)]
    {
        use std::path::Path;
        
        let lock_file = ".shortlinker.lock";
        
        if !Path::new(lock_file).exists() {
            print_warning!("未找到锁文件，服务器可能未运行");
            return;
        }
        
        print_warning!("Windows 平台不支持自动停止服务器");
        print_info!("请手动停止服务器进程，然后删除锁文件:");
        print_usage!("  del {}", lock_file);
        print_info!("或使用任务管理器终止 shortlinker.exe 进程");
    }
}

fn restart_server() {
    print_info!("正在重启 shortlinker 服务器...");
    
    #[cfg(unix)]
    {
        use std::fs;
        use std::path::Path;
        
        let pid_file = "shortlinker.pid";
        
        // 首先检查服务器是否在运行
        if Path::new(pid_file).exists() {
            match fs::read_to_string(pid_file) {
                Ok(pid_str) => {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        use nix::sys::signal::{self, Signal};
                        use nix::unistd::Pid;
                        
                        let server_pid = Pid::from_raw(pid as i32);
                        
                        // 检查进程是否存在
                        if signal::kill(server_pid, None).is_ok() {
                            print_info!("停止现有服务器进程 (PID: {})...", pid);
                            
                            // 发送 SIGTERM 信号
                            match signal::kill(server_pid, Signal::SIGTERM) {
                                Ok(_) => {
                                    print_info!("已发送停止信号，等待进程结束...");
                                    
                                    // 等待进程结束，最多等待 5 秒
                                    for i in 0..10 {
                                        std::thread::sleep(std::time::Duration::from_millis(500));
                                        if signal::kill(server_pid, None).is_err() {
                                            print_success!("服务器已停止");
                                            break;
                                        }
                                        if i == 9 {
                                            print_warning!("服务器未在预期时间内停止，尝试强制终止...");
                                            match signal::kill(server_pid, Signal::SIGKILL) {
                                                Ok(_) => print_success!("服务器已强制停止"),
                                                Err(e) => {
                                                    print_error!("无法强制停止服务器: {}", e);
                                                    return;
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    print_error!("无法停止服务器进程: {}", e);
                                    return;
                                }
                            }
                        } else {
                            print_info!("进程 {} 不存在，清理 PID 文件", pid);
                        }
                        
                        // 清理 PID 文件
                        let _ = fs::remove_file(pid_file);
                    }
                }
                Err(_) => {
                    print_info!("清理损坏的 PID 文件");
                    let _ = fs::remove_file(pid_file);
                }
            }
        } else {
            print_info!("未发现运行中的服务器");
        }
        
        // 等待一小段时间确保端口释放
        std::thread::sleep(std::time::Duration::from_millis(1000));
        
        print_info!("启动新的服务器进程...");
        print_usage!("  ./shortlinker");
        print_info!("或在后台运行:");
        print_usage!("  nohup ./shortlinker > shortlinker.log 2>&1 &");
    }
    
    #[cfg(windows)]
    {
        use std::path::Path;
        
        let lock_file = ".shortlinker.lock";
        
        if Path::new(lock_file).exists() {
            print_warning!("检测到锁文件，服务器可能正在运行");
            print_info!("Windows 平台需要手动重启服务器:");
            println!();
            print_info!("1. 使用任务管理器终止 shortlinker.exe 进程");
            print_info!("2. 删除锁文件:");
            print_usage!("   del {}", lock_file);
            print_info!("3. 重新启动服务器:");
            print_usage!("   shortlinker.exe");
        } else {
            print_info!("未发现运行中的服务器，直接启动:");
            print_usage!("  shortlinker.exe");
        }
    }
}
