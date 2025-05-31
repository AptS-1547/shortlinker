use crate::errors::{Result, ShortlinkerError};
use std::fs;

#[cfg(unix)]
pub fn notify_server() -> Result<()> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

    // Read the PID from file and send SIGUSR1 to the server process
    match fs::read_to_string("shortlinker.pid") {
        Ok(pid_str) => {
            let pid: i32 = pid_str
                .trim()
                .parse()
                .map_err(|e| ShortlinkerError::validation(format!("PID格式无效: {}", e)))?;
            signal::kill(Pid::from_raw(pid), Signal::SIGUSR1)
                .map_err(|e| ShortlinkerError::signal_operation(format!("发送信号失败: {}", e)))?;
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
pub fn notify_server() -> Result<()> {
    // Windows平台使用触发文件方式
    match fs::write("shortlinker.reload", "") {
        Ok(_) => {
            println!("已通知服务器重新加载配置");
            Ok(())
        }
        Err(e) => {
            println!("通知服务器失败: {}", e);
            Err(ShortlinkerError::file_operation(format!(
                "通知服务器失败: {}",
                e
            )))
        }
    }
}
