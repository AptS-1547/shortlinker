use super::CliError;
use colored::*;

pub struct ProcessManager;

impl ProcessManager {
    pub fn start_server() -> Result<(), CliError> {
        println!("{} 正在启动 shortlinker 服务器...", "ℹ".bold().blue());

        #[cfg(unix)]
        {
            use std::fs;
            use std::path::Path;

            let pid_file = "shortlinker.pid";

            if Path::new(pid_file).exists() {
                match fs::read_to_string(pid_file) {
                    Ok(old_pid_str) => {
                        if let Ok(old_pid) = old_pid_str.trim().parse::<u32>() {
                            use nix::sys::signal;
                            use nix::unistd::Pid;

                            if signal::kill(Pid::from_raw(old_pid as i32), None).is_ok() {
                                println!(
                                    "{} 服务器已在运行 (PID: {})",
                                    "⚠".bold().yellow(),
                                    old_pid
                                );
                                println!(
                                    "{} 如需重启，请先使用 'stop' 命令停止服务器",
                                    "ℹ".bold().blue()
                                );
                                return Ok(());
                            } else {
                                println!("{} 清理孤立的 PID 文件...", "ℹ".bold().blue());
                                let _ = fs::remove_file(pid_file);
                            }
                        }
                    }
                    Err(_) => {
                        let _ = fs::remove_file(pid_file);
                    }
                }
            }

            println!("{} 使用以下命令启动服务器:", "ℹ".bold().blue());
            println!("  {}", "./shortlinker".cyan());
            println!("{} 或在后台运行:", "ℹ".bold().blue());
            println!(
                "  {}",
                "nohup ./shortlinker > shortlinker.log 2>&1 &".cyan()
            );
        }

        #[cfg(windows)]
        {
            use std::path::Path;

            let lock_file = ".shortlinker.lock";

            if Path::new(lock_file).exists() {
                println!("{} 服务器可能已在运行", "⚠".bold().yellow());
                println!(
                    "{} 如确认服务器未运行，请删除锁文件: {}",
                    "ℹ".bold().blue(),
                    lock_file
                );
                println!("{} 然后重新启动服务器", "ℹ".bold().blue());
                return Ok(());
            }

            println!("{} 使用以下命令启动服务器:", "ℹ".bold().blue());
            println!("  {}", "shortlinker.exe".cyan());
        }

        Ok(())
    }

    pub fn stop_server() -> Result<(), CliError> {
        println!("{} 正在停止 shortlinker 服务器...", "ℹ".bold().blue());

        #[cfg(unix)]
        {
            use std::fs;
            use std::path::Path;

            let pid_file = "shortlinker.pid";

            if !Path::new(pid_file).exists() {
                println!("{} 未找到 PID 文件，服务器可能未运行", "⚠".bold().yellow());
                return Ok(());
            }

            match fs::read_to_string(pid_file) {
                Ok(pid_str) => {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        use nix::sys::signal::{self, Signal};
                        use nix::unistd::Pid;

                        let server_pid = Pid::from_raw(pid as i32);

                        if signal::kill(server_pid, None).is_err() {
                            println!("{} 进程 {} 不存在，清理 PID 文件", "⚠".bold().yellow(), pid);
                            let _ = fs::remove_file(pid_file);
                            return Ok(());
                        }

                        match signal::kill(server_pid, Signal::SIGTERM) {
                            Ok(_) => {
                                println!(
                                    "{} 已向服务器进程 {} 发送停止信号",
                                    "✓".bold().green(),
                                    pid
                                );

                                std::thread::sleep(std::time::Duration::from_secs(2));

                                if signal::kill(server_pid, None).is_ok() {
                                    println!(
                                        "{} 服务器进程仍在运行，尝试强制终止...",
                                        "⚠".bold().yellow()
                                    );
                                    match signal::kill(server_pid, Signal::SIGKILL) {
                                        Ok(_) => {
                                            println!("{} 服务器已强制停止", "✓".bold().green())
                                        }
                                        Err(e) => {
                                            return Err(CliError::ProcessError(format!(
                                                "无法强制停止服务器: {}",
                                                e
                                            )))
                                        }
                                    }
                                } else {
                                    println!("{} 服务器已正常停止", "✓".bold().green());
                                }

                                let _ = fs::remove_file(pid_file);
                            }
                            Err(e) => {
                                return Err(CliError::ProcessError(format!(
                                    "无法停止服务器进程: {}",
                                    e
                                )));
                            }
                        }
                    } else {
                        return Err(CliError::ProcessError("PID 文件格式无效".to_string()));
                    }
                }
                Err(e) => {
                    return Err(CliError::ProcessError(format!("无法读取 PID 文件: {}", e)));
                }
            }
        }

        #[cfg(windows)]
        {
            use std::path::Path;

            let lock_file = ".shortlinker.lock";

            if !Path::new(lock_file).exists() {
                println!("{} 未找到锁文件，服务器可能未运行", "⚠".bold().yellow());
                return Ok(());
            }

            println!("{} Windows 平台不支持自动停止服务器", "⚠".bold().yellow());
            println!(
                "{} 请手动停止服务器进程，然后删除锁文件:",
                "ℹ".bold().blue()
            );
            println!("  {}", format!("del {}", lock_file).cyan());
            println!(
                "{} 或使用任务管理器终止 shortlinker.exe 进程",
                "ℹ".bold().blue()
            );
        }

        Ok(())
    }

    pub fn restart_server() -> Result<(), CliError> {
        println!("{} 正在重启 shortlinker 服务器...", "ℹ".bold().blue());

        #[cfg(unix)]
        {
            // 先停止服务器
            if let Err(e) = Self::stop_server() {
                // 如果停止失败但不是因为服务器未运行，则返回错误
                if !e.to_string().contains("未找到 PID 文件") {
                    return Err(e);
                }
            }

            // 等待一小段时间确保端口释放
            std::thread::sleep(std::time::Duration::from_millis(1000));

            println!("{} 启动新的服务器进程...", "ℹ".bold().blue());
            println!("  {}", "./shortlinker".cyan());
            println!("{} 或在后台运行:", "ℹ".bold().blue());
            println!(
                "  {}",
                "nohup ./shortlinker > shortlinker.log 2>&1 &".cyan()
            );
        }

        #[cfg(windows)]
        {
            use std::path::Path;

            let lock_file = ".shortlinker.lock";

            if Path::new(lock_file).exists() {
                println!("{} 检测到锁文件，服务器可能正在运行", "⚠".bold().yellow());
                println!("{} Windows 平台需要手动重启服务器:", "ℹ".bold().blue());
                println!();
                println!(
                    "{} 1. 使用任务管理器终止 shortlinker.exe 进程",
                    "ℹ".bold().blue()
                );
                println!("{} 2. 删除锁文件:", "ℹ".bold().blue());
                println!("   {}", format!("del {}", lock_file).cyan());
                println!("{} 3. 重新启动服务器:", "ℹ".bold().blue());
                println!("   {}", "shortlinker.exe".cyan());
            } else {
                println!("{} 未发现运行中的服务器，直接启动:", "ℹ".bold().blue());
                println!("  {}", "shortlinker.exe".cyan());
            }
        }

        Ok(())
    }
}
