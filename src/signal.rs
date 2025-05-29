use std::fs;

#[cfg(unix)]
pub fn notify_server() -> Result<(), Box<dyn std::error::Error>> {
    use nix::sys::signal::{self, Signal};
    use nix::unistd::Pid;

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
pub fn notify_server() -> Result<(), Box<dyn std::error::Error>> {
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
