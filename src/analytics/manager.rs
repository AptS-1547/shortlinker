use dashmap::DashMap;
use once_cell::sync::Lazy;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tracing::{debug, trace};

use crate::analytics::ClickSink;

// 全局缓冲区，用于临时计数点击
pub static CLICK_BUFFER: Lazy<DashMap<String, usize>> = Lazy::new(DashMap::new);

// 全局缓冲区计数器，用于跟踪当前缓冲区中的点击数量
pub static CLICK_BUFFER_COUNTER: AtomicUsize = AtomicUsize::new(0);

// 全局更新锁，防止多线程同时更新缓冲区
pub static CLICK_UPDATE_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[derive(Clone)]
pub struct ClickManager {
    sink: Arc<dyn ClickSink>,
    flush_interval: Duration,
    max_clicks_before_flush: usize,
}

impl ClickManager {
    pub fn new(
        sink: Arc<dyn ClickSink>,
        flush_interval: Duration,
        max_clicks_before_flush: usize,
    ) -> Self {
        Self {
            sink,
            flush_interval,
            max_clicks_before_flush,
        }
    }

    /// 增加点击计数（线程安全，无锁）
    pub fn increment(&self, key: &str) {
        // 使用 DashMap 的原子操作来避免竞态条件
        let mut is_new_entry = false;
        CLICK_BUFFER
            .entry(key.to_string())
            .and_modify(|v| *v += 1)
            .or_insert_with(|| {
                is_new_entry = true;
                1
            });

        trace!("ClickManager: Incremented click for key: {}", key);

        // 只有新条目才增加计数器
        if is_new_entry {
            let current_size = CLICK_BUFFER_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;

            trace!("ClickManager: Current buffer size: {}", current_size);

            // 检查是否达到阈值，尝试获取锁进行刷盘
            if current_size >= self.max_clicks_before_flush {
                let sink = Arc::clone(&self.sink);
                tokio::spawn(async move {
                    if let Ok(_guard) = CLICK_UPDATE_LOCK.try_lock() {
                        Self::flush_inner(sink).await;
                        // guard 在此处自动释放
                    } else {
                        trace!("ClickManager: flush already in progress, skipping");
                    }
                });
            }
        }
    }

    /// 启动后台刷盘任务（作为异步方法运行）
    pub async fn start_background_task(&self) {
        loop {
            sleep(self.flush_interval).await;

            debug!("ClickManager: Triggering flush to storage");
            // 定期触发刷盘
            if let Ok(_guard) = CLICK_UPDATE_LOCK.try_lock() {
                trace!("ClickManager: Starting scheduled flush");
                let sink = Arc::clone(&self.sink);
                Self::flush_inner(sink).await;
                // guard 在此处自动释放
            } else {
                trace!("ClickManager: flush already in progress, skipping scheduled flush");
            }
        }
    }

    pub async fn flush(&self) {
        // 手动触发刷盘，等待获取锁
        debug!("ClickManager: Manual flush triggered");
        let _guard = CLICK_UPDATE_LOCK.lock().await;
        Self::flush_inner(Arc::clone(&self.sink)).await;
    }

    async fn flush_inner(sink: Arc<dyn ClickSink>) {
        let updates = CLICK_BUFFER
            .iter()
            .map(|entry| (entry.key().clone(), *entry.value()))
            .collect::<Vec<_>>();

        if updates.is_empty() {
            trace!("ClickManager: No clicks to flush");
            return;
        }

        let result = sink.flush_clicks(updates).await;

        if let Err(e) = result {
            trace!("ClickManager: flush_clicks failed: {}", e);
            // 保持缓冲区不变，稍后重试
            return;
        } else {
            CLICK_BUFFER.clear();
            CLICK_BUFFER_COUNTER.store(0, Ordering::Release);
            debug!("ClickManager: flush successful");
        }

        debug!("ClickManager: flush completed");
    }
}
