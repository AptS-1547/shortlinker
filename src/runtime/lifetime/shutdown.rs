use sea_orm::DatabaseConnection;
use tokio::signal;
use tracing::warn;

use crate::analytics::global::get_click_manager;

pub async fn listen_for_shutdown(db: &DatabaseConnection) {
    // 等待 Ctrl+C 信号
    match signal::ctrl_c().await {
        Ok(()) => {
            warn!("Shutdown signal received, flushing data...");
        }
        Err(e) => {
            warn!(
                "Failed to listen for Ctrl+C: {}. Proceeding with shutdown anyway.",
                e
            );
        }
    }

    // 刷新点击计数
    if let Some(manager) = get_click_manager() {
        manager.flush().await;
        warn!("ClickManager flushed successfully");
    } else {
        warn!("ClickManager is not initialized, skipping flush");
    }

    // 刷新待写入的 UserAgent 数据
    if let Some(store) = crate::services::get_user_agent_store() {
        match store.flush_pending(db).await {
            Ok(count) if count > 0 => {
                warn!("Flushed {} pending UserAgents on shutdown", count);
            }
            Err(e) => {
                warn!("Failed to flush UserAgents on shutdown: {}", e);
            }
            _ => {}
        }
    }

    crate::system::platform::cleanup_lockfile();
    warn!("Lockfile cleaned up, shutting down...");
}
