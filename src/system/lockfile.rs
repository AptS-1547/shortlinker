use std::fs;
use tracing::{error, info};

// Unix平台的PID文件管理
#[cfg(unix)]
pub fn init_lockfile() -> Result<(), std::io::Error> {
    use nix::sys::signal;
    use nix::unistd::Pid;
    use std::path::Path;
    use std::process;

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
                        std::process::exit(1);
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
        return Err(e);
    } else {
        info!("服务器 PID: {}", pid);
    }

    Ok(())
}

// Windows平台的锁文件管理
#[cfg(windows)]
pub fn init_lockfile() -> Result<(), std::io::Error> {
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
                return Err(e);
            }
        }
        Err(e) => {
            error!("无法创建锁文件: {}", e);
            return Err(e);
        }
    }

    Ok(())
}

// 清理锁文件
#[cfg(unix)]
pub fn cleanup_lockfile() {
    let pid_file = "shortlinker.pid";
    if let Err(e) = fs::remove_file(pid_file) {
        error!("无法删除PID文件: {}", e);
    } else {
        info!("已清理PID文件: {}", pid_file);
    }
}

#[cfg(windows)]
pub fn cleanup_lockfile() {
    let lock_file = ".shortlinker.lock";
    if let Err(e) = fs::remove_file(lock_file) {
        error!("无法删除锁文件: {}", e);
    } else {
        info!("已清理锁文件: {}", lock_file);
    }
}
