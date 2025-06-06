use super::CliError;
use colored::*;
pub use crate::structs::ProcessManager;

impl ProcessManager {
    pub fn start_server() -> Result<(), CliError> {
        println!("{} Starting shortlinker server...", "ℹ".bold().blue());

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
                                    "{} Server already running (PID: {})",
                                    "⚠".bold().yellow(),
                                    old_pid
                                );
                                println!(
                                    "{} To restart, first use the 'stop' command",
                                    "ℹ".bold().blue()
                                );
                                return Ok(());
                            } else {
                                println!("{} Cleaning up stale PID file...", "ℹ".bold().blue());
                                let _ = fs::remove_file(pid_file);
                            }
                        }
                    }
                    Err(_) => {
                        let _ = fs::remove_file(pid_file);
                    }
                }
            }

            println!("{} Start the server with:", "ℹ".bold().blue());
            println!("  {}", "./shortlinker".cyan());
            println!("{} Or run in the background:", "ℹ".bold().blue());
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
                println!("{} Server may already be running", "⚠".bold().yellow());
                println!(
                    "{} If the server is not running, delete the lock file: {}",
                    "ℹ".bold().blue(),
                    lock_file
                );
                println!("{} Then restart the server", "ℹ".bold().blue());
                return Ok(());
            }

            println!("{} Start the server with:", "ℹ".bold().blue());
            println!("  {}", "shortlinker.exe".cyan());
        }

        Ok(())
    }

    pub fn stop_server() -> Result<(), CliError> {
        println!("{} Stopping shortlinker server...", "ℹ".bold().blue());

        #[cfg(unix)]
        {
            use std::fs;
            use std::path::Path;

            let pid_file = "shortlinker.pid";

            if !Path::new(pid_file).exists() {
                println!(
                    "{} PID file not found, server may not be running",
                    "⚠".bold().yellow()
                );
                return Ok(());
            }

            match fs::read_to_string(pid_file) {
                Ok(pid_str) => {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        use nix::sys::signal::{self, Signal};
                        use nix::unistd::Pid;

                        let server_pid = Pid::from_raw(pid as i32);

                        if signal::kill(server_pid, None).is_err() {
                            println!(
                                "{} Process {} not found, cleaning PID file",
                                "⚠".bold().yellow(),
                                pid
                            );
                            let _ = fs::remove_file(pid_file);
                            return Ok(());
                        }

                        match signal::kill(server_pid, Signal::SIGTERM) {
                            Ok(_) => {
                                println!(
                                    "{} Sent stop signal to server process {}",
                                    "✓".bold().green(),
                                    pid
                                );

                                std::thread::sleep(std::time::Duration::from_secs(2));

                                if signal::kill(server_pid, None).is_ok() {
                                    println!(
                                        "{} Server process still running, trying to force kill...",
                                        "⚠".bold().yellow()
                                    );
                                    match signal::kill(server_pid, Signal::SIGKILL) {
                                        Ok(_) => {
                                            println!("{} Server force stopped", "✓".bold().green())
                                        }
                                        Err(e) => {
                                            return Err(CliError::ProcessError(format!(
                                                "无法强制停止服务器: {}",
                                                e
                                            )))
                                        }
                                    }
                                } else {
                                    println!("{} Server stopped gracefully", "✓".bold().green());
                                }

                                let _ = fs::remove_file(pid_file);
                            }
                            Err(e) => {
                                return Err(CliError::ProcessError(format!(
                                    "Failed to stop server process: {}",
                                    e
                                )));
                            }
                        }
                    } else {
                        return Err(CliError::ProcessError(
                            "Invalid PID file format".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    return Err(CliError::ProcessError(format!(
                        "Failed to read PID file: {}",
                        e
                    )));
                }
            }
        }

        #[cfg(windows)]
        {
            use std::path::Path;

            let lock_file = ".shortlinker.lock";

            if !Path::new(lock_file).exists() {
                println!(
                    "{} Lock file not found, server may not be running",
                    "⚠".bold().yellow()
                );
                return Ok(());
            }

            println!(
                "{} Windows does not support automatic stop",
                "⚠".bold().yellow()
            );
            println!(
                "{} Please stop the server process manually then delete the lock file:",
                "ℹ".bold().blue()
            );
            println!("  {}", format!("del {}", lock_file).cyan());
            println!(
                "{} Or terminate shortlinker.exe via Task Manager",
                "ℹ".bold().blue()
            );
        }

        Ok(())
    }

    pub fn restart_server() -> Result<(), CliError> {
        println!("{} Restarting shortlinker server...", "ℹ".bold().blue());

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

            println!("{} Starting new server process...", "ℹ".bold().blue());
            println!("  {}", "./shortlinker".cyan());
            println!("{} Or run in the background:", "ℹ".bold().blue());
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
                println!(
                    "{} Lock file detected, server might be running",
                    "⚠".bold().yellow()
                );
                println!("{} Windows requires manual restart:", "ℹ".bold().blue());
                println!();
                println!(
                    "{} 1. Terminate shortlinker.exe via Task Manager",
                    "ℹ".bold().blue()
                );
                println!("{} 2. Delete the lock file:", "ℹ".bold().blue());
                println!("   {}", format!("del {}", lock_file).cyan());
                println!("{} 3. Restart the server:", "ℹ".bold().blue());
                println!("   {}", "shortlinker.exe".cyan());
            } else {
                println!(
                    "{} No running server found, starting directly:",
                    "ℹ".bold().blue()
                );
                println!("  {}", "shortlinker.exe".cyan());
            }
        }

        Ok(())
    }
}
