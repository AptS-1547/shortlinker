use tokio::signal;
use tracing::warn;

use crate::analytics::global::get_click_manager;

pub async fn listen_for_shutdown() {
    // 等待 Ctrl+C 信号
    match signal::ctrl_c().await {
        Ok(()) => {
            warn!("Shutdown signal received, flushing click data...");
        }
        Err(e) => {
            warn!(
                "Failed to listen for Ctrl+C: {}. Proceeding with shutdown anyway.",
                e
            );
        }
    }

    // 调用点击管理器的 manual_flush
    let manager = get_click_manager();
    if let Some(manager) = manager {
        manager.flush().await;
    } else {
        warn!("ClickManager is not initialized, skipping flush");
    }

    warn!("ClickManager flushed successfully");

    crate::system::platform::cleanup_lockfile();
    warn!("Lockfile cleaned up, shutting down...");
}
