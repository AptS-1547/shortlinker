//! Status command - Query server status via IPC

use colored::Colorize;

use crate::interfaces::cli::CliError;
use crate::system::ipc::{self, IpcError, IpcResponse};

/// Display server status via IPC
pub async fn server_status() -> Result<(), CliError> {
    // Check if server is running
    if !ipc::is_server_running() {
        println!("{} Server is not running", "ℹ".bold().blue());
        return Ok(());
    }

    // Try to get detailed status
    match ipc::send_command(ipc::IpcCommand::GetStatus).await {
        Ok(IpcResponse::Status {
            version,
            uptime_secs,
            is_reloading,
            last_data_reload,
            last_config_reload,
            links_count,
        }) => {
            println!("{}", "Server Status".bold().green());
            println!("  {}:      {}", "Version".cyan(), version);
            println!(
                "  {}:       {}",
                "Uptime".cyan(),
                format_duration(uptime_secs)
            );
            println!(
                "  {}:    {}",
                "Reloading".cyan(),
                if is_reloading {
                    "yes".yellow()
                } else {
                    "no".green()
                }
            );

            if let Some(last_data) = last_data_reload {
                println!("  {}: {}", "Last data reload".cyan(), last_data.dimmed());
            }
            if let Some(last_config) = last_config_reload {
                println!(
                    "  {}: {}",
                    "Last config reload".cyan(),
                    last_config.dimmed()
                );
            }
            if links_count > 0 {
                println!("  {}:  {}", "Links count".cyan(), links_count);
            }

            Ok(())
        }
        Ok(IpcResponse::Error { code, message }) => Err(CliError::CommandError(format!(
            "Server error: {} - {}",
            code, message
        ))),
        Err(IpcError::ServerNotRunning) => {
            println!("{} Server is not running", "ℹ".bold().blue());
            Ok(())
        }
        Err(IpcError::Timeout) => Err(CliError::CommandError(
            "Connection timed out - server may be unresponsive".to_string(),
        )),
        Err(e) => Err(CliError::CommandError(format!(
            "Failed to get server status: {}",
            e
        ))),
        _ => Err(CliError::CommandError(
            "Unexpected response from server".to_string(),
        )),
    }
}

/// Format duration in human-readable form
fn format_duration(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
