//! Legacy signal module
//!
//! **DEPRECATED**: Use `system::platform` instead.
//! This module is kept for backward compatibility only.

use crate::errors::{Result, ShortlinkerError};
use std::fs;

#[deprecated(since = "0.2.1", note = "Use system::platform::notify_server instead")]
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
            Ok(())
        }
        Err(e) => Err(ShortlinkerError::notify_server(format!(
            "Failed to notify server: {}",
            e
        ))),
    }
}

#[deprecated(since = "0.2.1", note = "Use system::platform::notify_server instead")]
#[cfg(windows)]
pub fn notify_server() -> Result<()> {
    // On Windows use a trigger file
    match fs::write("shortlinker.reload", "") {
        Ok(_) => Ok(()),
        Err(e) => Err(ShortlinkerError::notify_server(format!(
            "Failed to notify server: {}",
            e
        ))),
    }
}
