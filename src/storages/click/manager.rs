use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::debug;

use crate::storages::click::ClickSink;

// 全局缓冲区，用于临时计数点击
pub static CLICK_BUFFER: Lazy<DashMap<String, usize>> = Lazy::new(DashMap::new);

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

    /// 启动后台刷盘任务
    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            loop {
                sleep(self.flush_interval).await;

                let updates: Vec<_> = CLICK_BUFFER
                    .iter()
                    .map(|entry| (entry.key().clone(), *entry.value()))
                    .collect();

                if updates.is_empty() {
                    continue;
                }

                CLICK_BUFFER.clear();

                if let Err(e) = self.sink.flush_clicks(updates).await {
                    debug!("ClickManager: flush_clicks failed: {}", e);
                }
            }
        });
    }
}
