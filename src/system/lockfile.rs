//! Legacy lockfile module
//!
//! **DEPRECATED**: Use `system::platform` instead.
//! This module is kept for backward compatibility only.

use std::fs;
use tracing::{error, info};

// Unix平台的PID文件管理
#[deprecated(since = "0.2.1", note = "Use system::platform::init_lockfile instead")]
#[cfg(unix)]
pub fn init_lockfile() -> Result<(), std::io::Error> {
    use nix::sys::signal;
    use nix::unistd::Pid;
    use std::path::Path;
    use std::process;
    use tracing::debug;

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
                        info!("Container restart detected, removing old PID file");
                        let _ = fs::remove_file(pid_file);
                    } else if signal::kill(Pid::from_raw(old_pid as i32), None).is_ok() {
                        error!("Server already running (PID: {}), stop it first", old_pid);
                        error!("You can stop it with:");
                        error!("  kill {}", old_pid);
                        std::process::exit(1);
                    } else {
                        // 进程已停止，清理旧的 PID 文件
                        info!("Stale PID file detected, cleaning up...");
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
        error!("Failed to write PID file: {}", e);
        return Err(e);
    } else {
        debug!("Server PID: {}", pid);
    }

    Ok(())
}

// Windows平台的锁文件管理
#[deprecated(since = "0.2.1", note = "Use system::platform::init_lockfile instead")]
#[cfg(windows)]
pub fn init_lockfile() -> Result<(), std::io::Error> {
    use std::io::{self, Write};
    use std::path::Path;

    let lock_file = ".shortlinker.lock";
    // Check if lock file already exists
    if Path::new(lock_file).exists() {
        error!("Server already running, stop it first");
        error!(
            "If the server is not running, delete the lock file: {}",
            lock_file
        );
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Server is already running",
        ));
    }

    match fs::File::create(lock_file) {
        Ok(mut file) => {
            if let Err(e) = writeln!(file, "Server is running") {
                error!("Failed to write lock file: {}", e);
                return Err(e);
            }
        }
        Err(e) => {
            error!("Failed to create lock file: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

// 清理锁文件
#[deprecated(since = "0.2.1", note = "Use system::platform::cleanup_lockfile instead")]
#[cfg(unix)]
pub fn cleanup_lockfile() {
    let pid_file = "shortlinker.pid";
    if let Err(e) = fs::remove_file(pid_file) {
        error!("Failed to delete PID file: {}", e);
    } else {
        info!("PID file cleaned: {}", pid_file);
    }
}

#[deprecated(since = "0.2.1", note = "Use system::platform::cleanup_lockfile instead")]
#[cfg(windows)]
pub fn cleanup_lockfile() {
    let lock_file = ".shortlinker.lock";
    if let Err(e) = fs::remove_file(lock_file) {
        error!("Failed to delete lock file: {}", e);
    } else {
        info!("Lock file cleaned: {}", lock_file);
    }
}
