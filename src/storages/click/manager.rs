use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::time::{sleep, Duration};
use tracing::debug;

use crate::storages::click::ClickSink;

// 全局缓冲区，用于临时计数点击
pub static CLICK_BUFFER: Lazy<DashMap<String, usize>> = Lazy::new(DashMap::new);

// 全局更新锁，防止多线程同时更新缓冲区
pub static CLICK_UPDATE_LOCK: AtomicBool = AtomicBool::new(false);

pub struct ClickManager {
    sink: Arc<dyn ClickSink>,
    flush_interval: Duration,
}

impl ClickManager {
    pub fn new(sink: Arc<dyn ClickSink>, flush_interval: Duration) -> Self {
        Self {
            sink,
            flush_interval,
        }
    }

    /// 增加点击计数（线程安全，无锁）
    pub fn increment(&self, key: &str) {
        *CLICK_BUFFER.entry(key.to_string()).or_insert(0) += 1;
    }

    /// 启动后台刷盘任务（作为异步方法运行）
    pub async fn start_background_task(&self) {
        loop {
            sleep(self.flush_interval).await;

            debug!("ClickManager: Triggering flush to storage");
            // 定期触发刷盘
            self.flush_inner().await;
        }
    }

    pub async fn flush(&self) {
        // 手动触发刷盘
        debug!("ClickManager: Manual flush triggered");
        self.flush_inner().await;
    }

    async fn flush_inner(&self) {
        if CLICK_UPDATE_LOCK.swap(true, Ordering::SeqCst) {
            debug!("ClickManager: flush already in progress, skipping");
            return; // 有其他 flush 正在进行
        }

        let result = {
            let updates = CLICK_BUFFER
                .iter()
                .map(|entry| (entry.key().clone(), *entry.value()))
                .collect::<Vec<_>>();

            if updates.is_empty() {
                debug!("ClickManager: No clicks to flush");
                CLICK_UPDATE_LOCK.store(false, Ordering::SeqCst);
                return;
            }
            CLICK_BUFFER.clear();

            self.sink.flush_clicks(updates).await
        };

        if let Err(e) = result {
            debug!("ClickManager: flush_clicks failed: {}", e);
        }

        CLICK_UPDATE_LOCK.store(false, Ordering::SeqCst);
        debug!("ClickManager: flush completed");
    }
}
