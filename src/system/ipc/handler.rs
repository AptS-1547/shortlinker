//! IPC command handler
//!
//! Processes incoming IPC commands and returns appropriate responses.

use std::sync::OnceLock;
use std::time::Instant;
use tracing::{debug, info, warn};

use super::types::{IpcCommand, IpcResponse};
use crate::system::reload::get_reload_coordinator;

/// Server start time for uptime calculation
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Initialize the server start time
///
/// Should be called once during server startup.
pub fn init_start_time() {
    START_TIME.get_or_init(Instant::now);
}

/// Get server uptime in seconds
fn get_uptime_secs() -> u64 {
    START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0)
}

/// Handle an IPC command and return a response
pub async fn handle_command(cmd: IpcCommand) -> IpcResponse {
    debug!("Handling IPC command: {:?}", cmd);

    match cmd {
        IpcCommand::Ping => IpcResponse::Pong {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_secs: get_uptime_secs(),
        },

        IpcCommand::Reload { target } => {
            info!("IPC reload request received: {:?}", target);

            let Some(coordinator) = get_reload_coordinator() else {
                warn!("ReloadCoordinator not initialized");
                return IpcResponse::Error {
                    code: "NOT_INITIALIZED".to_string(),
                    message: "ReloadCoordinator not initialized".to_string(),
                };
            };

            match coordinator.reload(target).await {
                Ok(result) => {
                    info!(
                        "IPC reload completed: {:?} in {}ms",
                        target, result.duration_ms
                    );
                    IpcResponse::ReloadResult {
                        success: true,
                        target,
                        duration_ms: result.duration_ms,
                        message: None,
                    }
                }
                Err(e) => {
                    warn!("IPC reload failed: {:?} - {}", target, e);
                    IpcResponse::ReloadResult {
                        success: false,
                        target,
                        duration_ms: 0,
                        message: Some(e.to_string()),
                    }
                }
            }
        }

        IpcCommand::GetStatus => {
            let coordinator = get_reload_coordinator();
            let status = coordinator.map(|c| c.status());

            IpcResponse::Status {
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_secs: get_uptime_secs(),
                is_reloading: status.as_ref().map(|s| s.is_reloading).unwrap_or(false),
                last_data_reload: status
                    .as_ref()
                    .and_then(|s| s.last_data_reload.as_ref())
                    .map(|r| r.finished_at.to_rfc3339()),
                last_config_reload: status
                    .as_ref()
                    .and_then(|s| s.last_config_reload.as_ref())
                    .map(|r| r.finished_at.to_rfc3339()),
                links_count: 0, // TODO: Get from storage/cache when available
            }
        }

        IpcCommand::Shutdown => {
            info!("IPC shutdown request received");
            // Note: Actual shutdown is handled by the caller
            // This just acknowledges the request
            IpcResponse::ShuttingDown
        }
    }
}
