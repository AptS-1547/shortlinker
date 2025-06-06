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
                .map_err(|e| ShortlinkerError::validation(format!("Invalid PID format: {}", e)))?;
            signal::kill(Pid::from_raw(pid), Signal::SIGUSR1).map_err(|e| {
                ShortlinkerError::signal_operation(format!("Failed to send signal: {}", e))
            })?;
            println!("Server reload notified");
            Ok(())
        }
        Err(_) => {
            println!("Warning: server process not found, please restart manually");
            Ok(())
        }
    }
}

#[cfg(windows)]
pub fn notify_server() -> Result<()> {
    // On Windows use a trigger file
    match fs::write("shortlinker.reload", "") {
        Ok(_) => {
            println!("Server reload notified");
            Ok(())
        }
        Err(e) => {
            println!("Failed to notify server: {}", e);
            Err(ShortlinkerError::file_operation(format!(
                "Failed to notify server: {}",
                e
            )))
        }
    }
}
