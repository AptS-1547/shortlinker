use sea_orm::DatabaseConnection;
use std::time::Duration;
use tokio::signal;
use tokio::time::timeout;
use tracing::{error, info, warn};

use crate::analytics::global::get_click_manager;

/// 关闭超时时间（秒）
const SHUTDOWN_TIMEOUT_SECS: u64 = 30;

/// 单个任务超时时间（秒）
const TASK_TIMEOUT_SECS: u64 = 10;

pub async fn listen_for_shutdown(db: &DatabaseConnection) {
    // 等待 Ctrl+C 信号
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received, flushing data...");
        }
        Err(e) => {
            warn!(
                "Failed to listen for Ctrl+C: {}. Proceeding with shutdown anyway.",
                e
            );
        }
    }

    // 将所有关闭任务包裹在超时内
    let shutdown_result = timeout(
        Duration::from_secs(SHUTDOWN_TIMEOUT_SECS),
        perform_shutdown_tasks(db),
    )
    .await;

    match shutdown_result {
        Ok(()) => {
            info!("All shutdown tasks completed successfully");
        }
        Err(_) => {
            error!(
                "Shutdown tasks timed out after {} seconds! Forcing exit.",
                SHUTDOWN_TIMEOUT_SECS
            );
            // 超时也要清理 lockfile
            crate::system::platform::cleanup_lockfile();
            std::process::exit(1);
        }
    }

    crate::system::platform::cleanup_lockfile();
    info!("Lockfile cleaned up, shutting down...");
}

/// 执行所有关闭任务（在超时内调用）
async fn perform_shutdown_tasks(db: &DatabaseConnection) {
    // 刷新点击计数（带重试）
    if let Some(manager) = get_click_manager() {
        const MAX_RETRIES: u32 = 3;
        let mut success = false;

        for attempt in 1..=MAX_RETRIES {
            match timeout(Duration::from_secs(TASK_TIMEOUT_SECS), manager.flush()).await {
                Ok(()) => {
                    info!(
                        "ClickManager flushed successfully (attempt {})",
                        attempt
                    );
                    success = true;
                    break;
                }
                Err(_) if attempt < MAX_RETRIES => {
                    warn!(
                        "ClickManager flush attempt {} timed out, retrying...",
                        attempt
                    );
                }
                Err(_) => {
                    error!(
                        "ClickManager flush failed after {} attempts (timed out)",
                        MAX_RETRIES
                    );
                }
            }
        }

        if !success {
            error!("ClickManager: data may be lost due to flush failure");
        }
    } else {
        info!("ClickManager is not initialized, skipping flush");
    }

    // 刷新待写入的 UserAgent 数据
    if let Some(store) = crate::services::get_user_agent_store() {
        match timeout(
            Duration::from_secs(TASK_TIMEOUT_SECS),
            store.flush_pending(db),
        )
        .await
        {
            Ok(Ok(count)) if count > 0 => {
                info!("Flushed {} pending UserAgents on shutdown", count);
            }
            Ok(Err(e)) => {
                error!("Failed to flush UserAgents on shutdown: {}", e);
            }
            Err(_) => {
                error!(
                    "UserAgent flush timed out after {} seconds",
                    TASK_TIMEOUT_SECS
                );
            }
            _ => {}
        }
    }
}
