//! 点击统计管理器
//!
//! 负责收集和刷新点击统计数据，支持：
//! - 高并发点击计数（使用 DashMap）
//! - 定时刷盘到存储后端
//! - 阈值触发刷盘

use dashmap::DashMap;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tracing::{debug, trace, warn};

use crate::analytics::ClickSink;

/// 点击缓冲区状态，封装所有可变状态
struct ClickBuffer {
    /// 点击计数缓冲区（使用 Arc<str> 减少克隆开销）
    data: DashMap<Arc<str>, usize>,
    /// 缓冲区中的总点击数（用于阈值判断）
    total_clicks: AtomicUsize,
    /// 刷盘锁，防止并发刷盘
    flush_lock: Mutex<()>,
}

impl ClickBuffer {
    fn new() -> Self {
        Self {
            data: DashMap::new(),
            total_clicks: AtomicUsize::new(0),
            flush_lock: Mutex::new(()),
        }
    }

    /// 增加点击计数
    fn increment(&self, key: &str) -> usize {
        // 先检查是否存在，避免不必要的字符串分配
        if let Some(mut entry) = self.data.get_mut(key) {
            *entry += 1;
            trace!("ClickBuffer: Incremented existing key: {}", key);
            return self.total_clicks.fetch_add(1, Ordering::Relaxed) + 1;
        }

        // 只有在需要插入新条目时才创建 Arc<str>
        self.data.insert(Arc::from(key), 1);
        trace!("ClickBuffer: Inserted new key: {}", key);

        self.total_clicks.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// 收集所有更新并清空缓冲区（逐个 remove 避免竞态）
    fn drain(&self) -> Vec<(String, usize)> {
        // 1. 收集所有 key（snapshot）
        let keys: Vec<Arc<str>> = self.data.iter().map(|r| r.key().clone()).collect();

        // 2. 逐个 remove（只删除 snapshot 中的 key，不影响窗口期新增）
        let mut updates = Vec::with_capacity(keys.len());
        let mut total_removed = 0;
        for key in keys {
            if let Some((k, v)) = self.data.remove(&key) {
                total_removed += v;
                updates.push((k.to_string(), v));
            }
        }

        // 3. 更新总计数
        if total_removed > 0 {
            self.total_clicks
                .fetch_sub(total_removed, Ordering::Release);
        }

        updates
    }

    /// 恢复数据到缓冲区（用于刷盘失败时的恢复）
    fn restore(&self, updates: Vec<(String, usize)>) {
        let mut restored_total = 0;
        for (k, v) in updates {
            *self.data.entry(Arc::from(k.as_str())).or_insert(0) += v;
            restored_total += v;
        }
        self.total_clicks
            .fetch_add(restored_total, Ordering::Relaxed);
    }

    /// 获取当前缓冲区总点击数
    fn total(&self) -> usize {
        self.total_clicks.load(Ordering::Relaxed)
    }
}

/// 点击管理器
///
/// 负责收集点击统计并定期刷盘到存储后端。
/// 状态完全封装在结构体内部，便于测试和多实例使用。
#[derive(Clone)]
pub struct ClickManager {
    /// 点击缓冲区（共享所有权）
    buffer: Arc<ClickBuffer>,
    /// 存储后端
    sink: Arc<dyn ClickSink>,
    /// 刷盘间隔
    flush_interval: Duration,
    /// 触发刷盘的最大点击数
    max_clicks_before_flush: usize,
}

impl ClickManager {
    pub fn new(
        sink: Arc<dyn ClickSink>,
        flush_interval: Duration,
        max_clicks_before_flush: usize,
    ) -> Self {
        Self {
            buffer: Arc::new(ClickBuffer::new()),
            sink,
            flush_interval,
            max_clicks_before_flush,
        }
    }

    /// 增加点击计数（线程安全，无锁）
    pub fn increment(&self, key: &str) {
        let current_size = self.buffer.increment(key);
        trace!("ClickManager: Current buffer size: {}", current_size);

        // 检查是否达到阈值，尝试获取锁进行刷盘
        if current_size >= self.max_clicks_before_flush {
            let buffer = Arc::clone(&self.buffer);
            let sink = Arc::clone(&self.sink);
            tokio::spawn(async move {
                if let Ok(_guard) = buffer.flush_lock.try_lock() {
                    Self::flush_buffer(&buffer, &sink).await;
                } else {
                    trace!("ClickManager: flush already in progress, skipping");
                }
            });
        }
    }

    /// 启动后台刷盘任务（作为异步方法运行）
    pub async fn start_background_task(&self) {
        loop {
            sleep(self.flush_interval).await;

            debug!("ClickManager: Triggering scheduled flush");
            // 定期触发刷盘
            if let Ok(_guard) = self.buffer.flush_lock.try_lock() {
                trace!("ClickManager: Starting scheduled flush");
                Self::flush_buffer(&self.buffer, &self.sink).await;
            } else {
                trace!("ClickManager: flush already in progress, skipping scheduled flush");
            }
        }
    }

    /// 手动触发刷盘（阻塞直到完成）
    pub async fn flush(&self) {
        debug!("ClickManager: Manual flush triggered");
        let _guard = self.buffer.flush_lock.lock().await;
        Self::flush_buffer(&self.buffer, &self.sink).await;
    }

    /// 执行实际的刷盘操作
    async fn flush_buffer(buffer: &ClickBuffer, sink: &Arc<dyn ClickSink>) {
        let updates = buffer.drain();

        if updates.is_empty() {
            trace!("ClickManager: No clicks to flush");
            return;
        }

        let count = updates.len();
        match sink.flush_clicks(updates.clone()).await {
            Ok(_) => {
                debug!("ClickManager: Successfully flushed {} entries", count);
            }
            Err(e) => {
                // 刷盘失败，恢复数据到 buffer
                buffer.restore(updates);
                warn!(
                    "ClickManager: flush_clicks failed: {}, {} entries restored to buffer",
                    e, count
                );
            }
        }
    }

    /// 获取当前缓冲区总点击数（用于监控）
    pub fn buffer_size(&self) -> usize {
        self.buffer.total()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockSink {
        flushed: std::sync::Mutex<Vec<(String, usize)>>,
    }

    impl MockSink {
        fn new() -> Self {
            Self {
                flushed: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn get_flushed(&self) -> Vec<(String, usize)> {
            self.flushed.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ClickSink for MockSink {
        async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
            self.flushed.lock().unwrap().extend(updates);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_increment_and_flush() {
        let sink = Arc::new(MockSink::new());
        let manager = ClickManager::new(
            Arc::clone(&sink) as Arc<dyn ClickSink>,
            Duration::from_secs(60),
            100,
        );

        manager.increment("key1");
        manager.increment("key1");
        manager.increment("key2");

        // buffer_size() 返回总点击数，不是唯一 key 数量
        assert_eq!(manager.buffer_size(), 3);

        manager.flush().await;

        assert_eq!(manager.buffer_size(), 0);
        let flushed = sink.get_flushed();
        assert_eq!(flushed.len(), 2); // 2 个唯一 key
    }
}
