//! 点击统计管理器
//!
//! 负责收集和刷新点击统计数据，支持：
//! - 高并发点击计数（使用 DashMap）
//! - 定时刷盘到存储后端
//! - 阈值触发刷盘
//! - 详细点击日志记录（可选）
//! - Channel 异步处理（避免热路径 spawn）

use crossbeam_channel::{Receiver, Sender, TrySendError};
use dashmap::DashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
};
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use tracing::{debug, trace, warn};

use crate::analytics::{ClickDetail, ClickSink, DetailedClickSink, RawClickEvent};

use crate::metrics_core::MetricsRecorder;

/// 点击缓冲区状态，封装所有可变状态
struct ClickBuffer {
    /// 点击计数缓冲区（使用 Arc<str> 减少克隆开销）
    data: DashMap<Arc<str>, usize>,
    /// 缓冲区中的总点击数（用于阈值判断）
    total_clicks: AtomicUsize,
    /// 刷盘锁，防止并发刷盘
    flush_lock: Mutex<()>,
    /// 是否有 flush 任务待处理（防止重复 spawn）
    flush_pending: AtomicBool,
}

impl ClickBuffer {
    fn new() -> Self {
        Self {
            data: DashMap::new(),
            total_clicks: AtomicUsize::new(0),
            flush_lock: Mutex::new(()),
            flush_pending: AtomicBool::new(false),
        }
    }

    /// 增加点击计数
    ///
    /// 使用 DashMap 的 entry API 实现原子性增加，无 TOCTOU 窗口。
    /// 每次新 key 都会分配 Arc，但代码更简洁且完全无竞态。
    fn increment(&self, key: &str) -> usize {
        self.data
            .entry(Arc::from(key))
            .and_modify(|v| *v += 1)
            .or_insert(1);

        trace!("ClickBuffer: Incremented key: {}", key);

        // 使用 AcqRel 确保与其他线程的操作正确同步
        self.total_clicks.fetch_add(1, Ordering::AcqRel) + 1
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

        // 3. 更新总计数（使用 AcqRel 确保一致性）
        if total_removed > 0 {
            self.total_clicks
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |current| {
                    Some(current.saturating_sub(total_removed))
                })
                .ok();
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
            .fetch_add(restored_total, Ordering::AcqRel);
    }

    /// 获取当前缓冲区总点击数
    fn total(&self) -> usize {
        self.total_clicks.load(Ordering::Acquire)
    }
}

/// 详细点击日志缓冲区
struct DetailedBuffer {
    /// 详细点击日志缓冲区
    data: DashMap<u64, ClickDetail>,
    /// 下一个 ID
    next_id: AtomicU64,
    /// 当前条目数（用于阈值判断）
    entry_count: AtomicUsize,
    /// 刷盘锁
    flush_lock: Mutex<()>,
    /// 是否有 flush 任务待处理（防止重复 spawn）
    flush_pending: AtomicBool,
}

impl DetailedBuffer {
    fn new() -> Self {
        Self {
            data: DashMap::new(),
            next_id: AtomicU64::new(0),
            entry_count: AtomicUsize::new(0),
            flush_lock: Mutex::new(()),
            flush_pending: AtomicBool::new(false),
        }
    }

    /// 添加详细点击日志，返回当前缓冲区大小
    fn push(&self, detail: ClickDetail) -> usize {
        let id = self.next_id.fetch_add(1, Ordering::AcqRel);
        self.data.insert(id, detail);
        self.entry_count.fetch_add(1, Ordering::AcqRel) + 1
    }

    /// 收集所有日志并清空缓冲区
    fn drain(&self) -> Vec<ClickDetail> {
        let keys: Vec<u64> = self.data.iter().map(|r| *r.key()).collect();
        let mut details = Vec::with_capacity(keys.len());
        for key in keys {
            if let Some((_, detail)) = self.data.remove(&key) {
                details.push(detail);
            }
        }
        // 重置计数器（使用 Release 确保其他线程看到正确的值）
        self.entry_count.store(0, Ordering::Release);
        details
    }

    /// 恢复数据到缓冲区（不调用 push 避免重复计数）
    fn restore(&self, details: Vec<ClickDetail>) {
        let count = details.len();
        for detail in details {
            let id = self.next_id.fetch_add(1, Ordering::AcqRel);
            self.data.insert(id, detail);
        }
        self.entry_count.fetch_add(count, Ordering::AcqRel);
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
    /// 详细日志缓冲区（可选）
    detailed_buffer: Option<Arc<DetailedBuffer>>,
    /// 详细日志 Sink（可选）
    detailed_sink: Option<Arc<dyn DetailedClickSink>>,
    /// 原始事件 channel sender（用于异步处理详细日志，使用 crossbeam 高性能 channel）
    raw_event_tx: Option<Sender<RawClickEvent>>,
    /// Metrics recorder for dependency injection
    metrics: Arc<dyn MetricsRecorder>,
}

impl ClickManager {
    pub fn new(
        sink: Arc<dyn ClickSink>,
        flush_interval: Duration,
        max_clicks_before_flush: usize,
        metrics: Arc<dyn MetricsRecorder>,
    ) -> Self {
        Self {
            buffer: Arc::new(ClickBuffer::new()),
            sink,
            flush_interval,
            max_clicks_before_flush,
            detailed_buffer: None,
            detailed_sink: None,
            raw_event_tx: None,
            metrics,
        }
    }

    /// 创建带详细日志支持的点击管理器
    /// 返回 (ClickManager, Receiver) 以便启动后台处理任务
    pub fn with_detailed_logging(
        sink: Arc<dyn ClickSink>,
        detailed_sink: Arc<dyn DetailedClickSink>,
        flush_interval: Duration,
        max_clicks_before_flush: usize,
        metrics: Arc<dyn MetricsRecorder>,
    ) -> (Self, Receiver<RawClickEvent>) {
        // 使用 crossbeam bounded channel，容量 10000
        let (tx, rx) = crossbeam_channel::bounded(10000);

        let manager = Self {
            buffer: Arc::new(ClickBuffer::new()),
            sink,
            flush_interval,
            max_clicks_before_flush,
            detailed_buffer: Some(Arc::new(DetailedBuffer::new())),
            detailed_sink: Some(detailed_sink),
            raw_event_tx: Some(tx),
            metrics,
        };

        (manager, rx)
    }

    /// 检查是否启用了详细日志
    pub fn is_detailed_logging_enabled(&self) -> bool {
        self.detailed_buffer.is_some() && self.detailed_sink.is_some()
    }

    /// 记录详细点击信息（如果启用）
    ///
    /// 同时增加 click_count 和记录详细日志
    pub fn record_detailed(&self, detail: ClickDetail) {
        // 1. 始终增加 click_count（现有逻辑）
        self.increment(&detail.code);

        // 2. 如果启用详细日志，写入 detailed_buffer
        if let Some(ref buffer) = self.detailed_buffer {
            let current_size = buffer.push(detail);
            trace!(
                "ClickManager: Detailed log recorded, buffer size: {}",
                current_size
            );

            // 阈值触发刷盘
            if current_size >= self.max_clicks_before_flush
                && buffer
                    .flush_pending
                    .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
                    .is_ok()
            {
                let buffer = Arc::clone(buffer);
                let sink = Arc::clone(self.detailed_sink.as_ref().unwrap());
                tokio::spawn(async move {
                    if let Ok(_guard) = buffer.flush_lock.try_lock() {
                        Self::flush_detailed_buffer(&buffer, &sink).await;
                    } else {
                        trace!("ClickManager: detailed flush already in progress, skipping");
                    }
                    buffer.flush_pending.store(false, Ordering::Release);
                });
            }
        }
    }

    /// 发送原始点击事件到 channel（热路径调用，非阻塞）
    ///
    /// 返回 true 表示发送成功，false 表示 channel 已满或未启用
    #[inline]
    pub fn send_raw_event(&self, event: RawClickEvent) -> bool {
        // 始终增加 click_count
        self.increment(&event.code);

        // 尝试发送到 channel（crossbeam try_send）
        if let Some(ref tx) = self.raw_event_tx {
            match tx.try_send(event) {
                Ok(()) => true,
                Err(TrySendError::Full(_)) => {
                    warn!("ClickManager: Event channel full, dropping event");
                    self.metrics.inc_clicks_channel_dropped("full");
                    false
                }
                Err(TrySendError::Disconnected(_)) => {
                    warn!("ClickManager: Event channel disconnected");
                    self.metrics.inc_clicks_channel_dropped("disconnected");
                    false
                }
            }
        } else {
            false
        }
    }

    /// 获取 channel sender 的克隆（用于外部直接发送）
    pub fn get_event_sender(&self) -> Option<Sender<RawClickEvent>> {
        self.raw_event_tx.clone()
    }

    /// 增加点击计数（线程安全，无锁）
    pub fn increment(&self, key: &str) {
        let current_size = self.buffer.increment(key);
        trace!("ClickManager: Current buffer size: {}", current_size);

        // 检查是否达到阈值，尝试触发刷盘
        if current_size >= self.max_clicks_before_flush {
            // 使用 compare_exchange 防止任务风暴：
            // 只有成功将 flush_pending 从 false 设为 true 的线程才 spawn
            if self
                .buffer
                .flush_pending
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
                .is_ok()
            {
                let buffer = Arc::clone(&self.buffer);
                let sink = Arc::clone(&self.sink);
                let metrics = Arc::clone(&self.metrics);
                tokio::spawn(async move {
                    let success = if let Ok(_guard) = buffer.flush_lock.try_lock() {
                        Self::flush_buffer_with_trigger(&buffer, &sink, "threshold", &metrics).await
                    } else {
                        trace!("ClickManager: flush already in progress, skipping");
                        true // 跳过也算成功，不需要退避
                    };

                    // 失败时延迟 5 秒再重置标志，实现退避
                    if !success {
                        trace!("ClickManager: flush failed, backing off for 5 seconds");
                        sleep(Duration::from_secs(5)).await;
                    }
                    buffer.flush_pending.store(false, Ordering::Release);
                });
            }
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
                Self::flush_buffer(&self.buffer, &self.sink, &self.metrics).await;
            } else {
                trace!("ClickManager: flush already in progress, skipping scheduled flush");
            }

            // 刷新详细日志
            if let (Some(detailed_buffer), Some(detailed_sink)) =
                (&self.detailed_buffer, &self.detailed_sink)
                && let Ok(_guard) = detailed_buffer.flush_lock.try_lock()
            {
                Self::flush_detailed_buffer(detailed_buffer, detailed_sink).await;
            }
        }
    }

    /// 启动原始事件处理器（消费 crossbeam channel 并生成 ClickDetail）
    ///
    /// 需要传入事件处理函数，用于将 RawClickEvent 转换为 ClickDetail
    pub async fn start_event_processor<F>(&self, rx: Receiver<RawClickEvent>, process_fn: F)
    where
        F: Fn(RawClickEvent) -> ClickDetail + Send + 'static,
    {
        debug!("ClickManager: Starting event processor");

        // crossbeam channel 的 recv 是阻塞的，需要在 blocking task 中运行
        // 或者用 try_recv + yield
        loop {
            // 使用 try_recv 避免阻塞 tokio runtime
            match rx.try_recv() {
                Ok(event) => {
                    let detail = process_fn(event);

                    // 直接写入 detailed_buffer（不再调用 record_detailed 避免重复 increment）
                    if let Some(ref buffer) = self.detailed_buffer {
                        let current_size = buffer.push(detail);

                        // 阈值触发刷盘
                        if current_size >= self.max_clicks_before_flush
                            && buffer
                                .flush_pending
                                .compare_exchange(false, true, Ordering::SeqCst, Ordering::Relaxed)
                                .is_ok()
                        {
                            let buffer = Arc::clone(buffer);
                            let sink = Arc::clone(self.detailed_sink.as_ref().unwrap());
                            tokio::spawn(async move {
                                if let Ok(_guard) = buffer.flush_lock.try_lock() {
                                    Self::flush_detailed_buffer(&buffer, &sink).await;
                                }
                                buffer.flush_pending.store(false, Ordering::Release);
                            });
                        }
                    }
                }
                Err(crossbeam_channel::TryRecvError::Empty) => {
                    // Channel 空，短暂休眠避免忙等（1ms 平衡延迟和 CPU）
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
                Err(crossbeam_channel::TryRecvError::Disconnected) => {
                    debug!("ClickManager: Event channel disconnected, stopping processor");
                    break;
                }
            }
        }

        debug!("ClickManager: Event processor stopped (channel closed)");
    }

    /// 手动触发刷盘（阻塞直到完成）
    pub async fn flush(&self) {
        debug!("ClickManager: Manual flush triggered");
        let _guard = self.buffer.flush_lock.lock().await;
        Self::flush_buffer_with_trigger(&self.buffer, &self.sink, "manual", &self.metrics).await;

        // 刷新详细日志
        if let (Some(detailed_buffer), Some(detailed_sink)) =
            (&self.detailed_buffer, &self.detailed_sink)
        {
            let _guard = detailed_buffer.flush_lock.lock().await;
            Self::flush_detailed_buffer(detailed_buffer, detailed_sink).await;
        }
    }

    /// 执行实际的刷盘操作
    async fn flush_buffer(
        buffer: &ClickBuffer,
        sink: &Arc<dyn ClickSink>,
        metrics: &Arc<dyn MetricsRecorder>,
    ) -> bool {
        Self::flush_buffer_with_trigger(buffer, sink, "interval", metrics).await
    }

    /// 执行实际的刷盘操作（带触发类型标记）
    ///
    /// 返回 true 表示成功，false 表示失败
    async fn flush_buffer_with_trigger(
        buffer: &ClickBuffer,
        sink: &Arc<dyn ClickSink>,
        trigger: &str,
        metrics: &Arc<dyn MetricsRecorder>,
    ) -> bool {
        let updates = buffer.drain();

        // Update buffer size gauge after drain
        metrics.set_clicks_buffer_entries(buffer.data.len() as f64);

        if updates.is_empty() {
            trace!("ClickManager: No clicks to flush");
            return true; // 没有数据也算成功
        }

        let count = updates.len();
        match sink.flush_clicks(updates.clone()).await {
            Ok(_) => {
                debug!("ClickManager: Successfully flushed {} entries", count);
                metrics.inc_clicks_flush(trigger, "success");
                true
            }
            Err(e) => {
                // 刷盘失败，恢复数据到 buffer
                buffer.restore(updates);
                warn!(
                    "ClickManager: flush_clicks failed: {}, {} entries restored to buffer",
                    e, count
                );
                metrics.inc_clicks_flush(trigger, "failed");
                // Update gauge again after restore
                metrics.set_clicks_buffer_entries(buffer.data.len() as f64);
                false
            }
        }
    }

    /// 执行详细日志刷盘操作（分批插入，避免超出 SQL 变量限制）
    async fn flush_detailed_buffer(buffer: &DetailedBuffer, sink: &Arc<dyn DetailedClickSink>) {
        let details = buffer.drain();

        if details.is_empty() {
            trace!("ClickManager: No detailed logs to flush");
            return;
        }

        let total_count = details.len();
        const BATCH_SIZE: usize = 500;
        let mut failed = Vec::new();
        let mut success_count = 0;

        for chunk in details.chunks(BATCH_SIZE) {
            match sink.log_clicks_batch(chunk.to_vec()).await {
                Ok(_) => {
                    success_count += chunk.len();
                }
                Err(e) => {
                    warn!(
                        "ClickManager: log_clicks_batch failed: {}, {} entries will be restored",
                        e,
                        chunk.len()
                    );
                    failed.extend(chunk.iter().cloned());
                }
            }
        }

        if success_count > 0 {
            debug!(
                "ClickManager: Successfully flushed {} detailed log entries",
                success_count
            );
        }

        if !failed.is_empty() {
            warn!(
                "ClickManager: {} of {} entries failed, restoring to buffer",
                failed.len(),
                total_count
            );
            buffer.restore(failed);
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

    use crate::metrics_core::NoopMetrics;

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

        fn total_clicks(&self) -> usize {
            self.flushed.lock().unwrap().iter().map(|(_, v)| v).sum()
        }
    }

    #[async_trait]
    impl ClickSink for MockSink {
        async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
            self.flushed.lock().unwrap().extend(updates);
            Ok(())
        }
    }

    fn create_test_manager(sink: Arc<dyn ClickSink>, max_clicks: usize) -> ClickManager {
        ClickManager::new(
            sink,
            Duration::from_secs(60),
            max_clicks,
            NoopMetrics::arc(),
        )
    }

    #[tokio::test]
    async fn test_increment_and_flush() {
        let sink = Arc::new(MockSink::new());
        let manager = create_test_manager(Arc::clone(&sink) as Arc<dyn ClickSink>, 100);

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

    /// 测试并发 increment 不会丢失点击
    #[tokio::test]
    async fn test_concurrent_increment() {
        let sink = Arc::new(MockSink::new());
        let manager = Arc::new(create_test_manager(
            Arc::clone(&sink) as Arc<dyn ClickSink>,
            100000, // 高阈值，避免自动刷盘
        ));

        const NUM_THREADS: usize = 10;
        const INCREMENTS_PER_THREAD: usize = 1000;

        let mut handles = vec![];
        for _ in 0..NUM_THREADS {
            let mgr = Arc::clone(&manager);
            handles.push(tokio::spawn(async move {
                for _ in 0..INCREMENTS_PER_THREAD {
                    mgr.increment("shared_key");
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // 验证 buffer 中的计数正确
        assert_eq!(manager.buffer_size(), NUM_THREADS * INCREMENTS_PER_THREAD);

        manager.flush().await;

        // 验证刷盘后的数据正确
        assert_eq!(sink.total_clicks(), NUM_THREADS * INCREMENTS_PER_THREAD);
    }

    /// 测试并发 increment + drain 不会丢失数据
    #[tokio::test]
    async fn test_concurrent_increment_and_drain() {
        let sink = Arc::new(MockSink::new());
        let manager = Arc::new(create_test_manager(
            Arc::clone(&sink) as Arc<dyn ClickSink>,
            100000, // 高阈值，避免自动刷盘
        ));

        const NUM_THREADS: usize = 10;
        const INCREMENTS_PER_THREAD: usize = 1000;
        const NUM_FLUSHES: usize = 5;

        // 启动 increment 线程
        let mut handles = vec![];
        for _ in 0..NUM_THREADS {
            let mgr = Arc::clone(&manager);
            handles.push(tokio::spawn(async move {
                for _ in 0..INCREMENTS_PER_THREAD {
                    mgr.increment("shared_key");
                    // 偶尔 yield，增加与 drain 交错的机会
                    if rand::random::<u8>() < 10 {
                        tokio::task::yield_now().await;
                    }
                }
            }));
        }

        // 启动 flush 线程
        let mgr_flush = Arc::clone(&manager);
        let flush_handle = tokio::spawn(async move {
            for _ in 0..NUM_FLUSHES {
                tokio::time::sleep(Duration::from_millis(10)).await;
                mgr_flush.flush().await;
            }
        });

        // 等待所有 increment 完成
        for handle in handles {
            handle.await.unwrap();
        }
        flush_handle.await.unwrap();

        // 最后一次 flush 确保所有数据都写入
        manager.flush().await;

        // 验证总点击数 = 已刷盘 + buffer 中剩余
        let flushed = sink.total_clicks();
        let remaining = manager.buffer_size();
        assert_eq!(
            flushed + remaining,
            NUM_THREADS * INCREMENTS_PER_THREAD,
            "flushed={}, remaining={}, expected={}",
            flushed,
            remaining,
            NUM_THREADS * INCREMENTS_PER_THREAD
        );
    }
}
