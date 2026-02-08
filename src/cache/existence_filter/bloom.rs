use async_trait::async_trait;
use bloomfilter::Bloom;
use futures_util::StreamExt;
use parking_lot::{Mutex, RwLock};
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

use crate::cache::ExistenceFilter;
use crate::declare_existence_filter_plugin;
use crate::errors::{Result, ShortlinkerError};
use futures_util::stream::Stream;

declare_existence_filter_plugin!("bloom", BloomExistenceFilterPlugin);

pub struct BloomExistenceFilterPlugin {
    inner: Arc<RwLock<Bloom<str>>>,
    /// rebuild 期间收集新增 key 的 buffer。
    /// Some = 正在重建，set() 会同时写入 buffer
    /// None = 未在重建
    rebuild_buffer: Mutex<Option<Vec<String>>>,
}

impl Default for BloomExistenceFilterPlugin {
    fn default() -> Self {
        Self::new().expect("Failed to create default BloomExistenceFilterPlugin")
    }
}

impl BloomExistenceFilterPlugin {
    pub fn new() -> Result<Self> {
        // 使用最小初始容量，因为 startup.rs 中的 rebuild_all() 会立即用实际数量替换
        let bloom = Bloom::new_for_fp_rate(100, 0.001).map_err(|e| {
            ShortlinkerError::cache_connection(format!("Failed to create bloom filter: {e}"))
        })?;
        Ok(Self {
            inner: Arc::new(RwLock::new(bloom)),
            rebuild_buffer: Mutex::new(None),
        })
    }
}

/// 分段预留策略，计算 Bloom Filter 实际容量
/// - < 5000: 预留 50%（小规模需要更多余量）
/// - 5000-100000: 预留 20%
/// - > 100000: 预留 10%（最多 100 万）
fn calculate_capacity(count: usize) -> usize {
    let reserve = if count < 5000 {
        count / 2
    } else if count < 100000 {
        count / 5
    } else {
        (count / 10).min(1_000_000)
    };
    count + reserve.max(1000) // 最少预留 1000
}

#[async_trait]
impl ExistenceFilter for BloomExistenceFilterPlugin {
    async fn check(&self, key: &str) -> bool {
        let bloom = self.inner.read();
        bloom.check(key)
    }

    async fn set(&self, key: &str) {
        // 锁顺序：buffer lock → inner write lock（与 rebuild_streaming 一致，防止死锁）
        let mut buffer_guard = self.rebuild_buffer.lock();
        self.inner.write().set(key);
        if let Some(ref mut buffer) = *buffer_guard {
            buffer.push(key.to_string());
        }
    }

    async fn bulk_set(&self, keys: &[String]) {
        let mut bloom = self.inner.write();
        for key in keys {
            bloom.set(key);
        }
        debug!("Bulk inserted {} keys into bloom filter", keys.len());
    }

    async fn clear(&self, count: usize, fp_rate: f64) -> Result<()> {
        let mut bloom = self.inner.write();
        let capacity = calculate_capacity(count);
        *bloom = Bloom::new_for_fp_rate(capacity, fp_rate).map_err(|e| {
            ShortlinkerError::cache_connection(format!("Failed to clear bloom filter: {e}"))
        })?;
        debug!(
            "Bloom filter cleared with capacity: {} (count: {} + reserve: {}), fp_rate: {}",
            capacity,
            count,
            capacity - count,
            fp_rate
        );
        Ok(())
    }

    /// 在锁外构建完整的新 Bloom Filter，然后原子交换。
    /// 读取端看到的要么是旧的完整 Bloom，要么是新的完整 Bloom，永远不会看到空 Bloom。
    /// rebuild 期间通过 buffer 捕获并发 set() 写入的 key，确保零丢失。
    async fn rebuild(&self, keys: &[String], count: usize, fp_rate: f64) -> Result<()> {
        // 启用 buffer，捕获 rebuild 期间的并发写入
        *self.rebuild_buffer.lock() = Some(Vec::new());

        let capacity = calculate_capacity(count);
        let mut new_bloom = Bloom::new_for_fp_rate(capacity, fp_rate).map_err(|e| {
            // 构建失败时关闭 buffer
            *self.rebuild_buffer.lock() = None;
            ShortlinkerError::cache_connection(format!("Failed to rebuild bloom filter: {e}"))
        })?;
        for key in keys {
            new_bloom.set(key.as_str());
        }

        // 持 buffer lock → drain buffer → 交换 → 关闭 buffer
        let buffered_count;
        {
            let mut buffer_guard = self.rebuild_buffer.lock();
            if let Some(ref pending) = *buffer_guard {
                buffered_count = pending.len();
                for key in pending {
                    new_bloom.set(key.as_str());
                }
            } else {
                buffered_count = 0;
            }
            *self.inner.write() = new_bloom;
            *buffer_guard = None;
        }

        debug!(
            "Bloom filter rebuilt atomically with {} keys ({} from buffer), capacity: {} (count: {} + reserve: {}), fp_rate: {}",
            keys.len() + buffered_count,
            buffered_count,
            capacity,
            count,
            capacity - count,
            fp_rate
        );
        Ok(())
    }

    /// 流式重建 Bloom Filter：用 count 预分配，从 Stream 逐批加载，最后原子交换。
    /// 内存占用 O(batch_size) 而非 O(total_keys)，避免大数据量 OOM。
    /// rebuild 期间通过 buffer 捕获并发 set() 写入的 key，确保零丢失。
    async fn rebuild_streaming(
        &self,
        count: usize,
        fp_rate: f64,
        stream: Pin<Box<dyn Stream<Item = Result<Vec<String>>> + Send>>,
    ) -> Result<()> {
        // 启用 buffer，捕获 rebuild 期间的并发写入
        *self.rebuild_buffer.lock() = Some(Vec::new());

        let capacity = calculate_capacity(count);
        let mut new_bloom = Bloom::new_for_fp_rate(capacity, fp_rate).map_err(|e| {
            *self.rebuild_buffer.lock() = None;
            ShortlinkerError::cache_connection(format!(
                "Failed to create bloom filter for streaming rebuild: {e}"
            ))
        })?;

        let mut loaded: usize = 0;
        let mut stream = stream;

        while let Some(batch_result) = stream.next().await {
            match batch_result {
                Ok(batch) => {
                    for key in &batch {
                        new_bloom.set(key.as_str());
                    }
                    loaded += batch.len();
                }
                Err(e) => {
                    // 流式读取失败，关闭 buffer，旧 Bloom 保持不变
                    *self.rebuild_buffer.lock() = None;
                    return Err(e);
                }
            }
        }

        // 持 buffer lock → drain buffer → 交换 → 关闭 buffer
        let buffered_count;
        {
            let mut buffer_guard = self.rebuild_buffer.lock();
            if let Some(ref pending) = *buffer_guard {
                buffered_count = pending.len();
                for key in pending {
                    new_bloom.set(key.as_str());
                }
            } else {
                buffered_count = 0;
            }
            *self.inner.write() = new_bloom;
            *buffer_guard = None;
        }

        debug!(
            "Bloom filter rebuilt (streaming) with {} keys ({} from buffer), capacity: {} (count: {} + reserve: {}), fp_rate: {}",
            loaded + buffered_count,
            buffered_count,
            capacity,
            count,
            capacity - count,
            fp_rate
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_empty_returns_false() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();
        assert!(!filter.check("nonexistent").await);
    }

    #[tokio::test]
    async fn test_set_and_check() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();

        filter.set("test_key").await;
        assert!(filter.check("test_key").await);

        // 未设置的 key 应该返回 false（大概率）
        assert!(!filter.check("other_key").await);
    }

    #[tokio::test]
    async fn test_bulk_set() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();
        let keys: Vec<String> = (0..100).map(|i| format!("key_{}", i)).collect();

        filter.bulk_set(&keys).await;

        // 所有设置的 key 都应该存在
        for key in &keys {
            assert!(filter.check(key).await, "Key {} should exist", key);
        }
    }

    #[tokio::test]
    async fn test_clear_resets_filter() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();

        filter.set("test_key").await;
        assert!(filter.check("test_key").await);

        filter.clear(1000, 0.001).await.unwrap();

        // 清除后，之前的 key 应该不存在了
        assert!(!filter.check("test_key").await);
    }

    #[tokio::test]
    async fn test_rebuild_replaces_filter_atomically() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();

        // 先插入一些旧 key
        filter.set("old_key_1").await;
        filter.set("old_key_2").await;
        assert!(filter.check("old_key_1").await);

        // 用新 key 列表 rebuild
        let new_keys = vec!["new_key_a".to_string(), "new_key_b".to_string()];
        filter
            .rebuild(&new_keys, new_keys.len(), 0.001)
            .await
            .unwrap();

        // 旧 key 应该不存在了
        assert!(!filter.check("old_key_1").await);
        assert!(!filter.check("old_key_2").await);

        // 新 key 应该存在
        assert!(filter.check("new_key_a").await);
        assert!(filter.check("new_key_b").await);
    }

    #[tokio::test]
    async fn test_unicode_key() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();

        filter.set("中文键").await;
        assert!(filter.check("中文键").await);

        filter.set("🎉emoji").await;
        assert!(filter.check("🎉emoji").await);
    }

    #[tokio::test]
    async fn test_false_positive_rate_within_bounds() {
        let filter = BloomExistenceFilterPlugin::new().unwrap();

        // 重新配置为足够大的容量（初始容量只有 100）
        filter.clear(1000, 0.001).await.unwrap();

        // 插入 1000 个 key
        let keys: Vec<String> = (0..1000).map(|i| format!("existing_{}", i)).collect();
        filter.bulk_set(&keys).await;

        // 测试 10000 个不存在的 key，统计误报率
        let mut false_positives = 0;
        for i in 0..10000 {
            if filter.check(&format!("nonexistent_{}", i)).await {
                false_positives += 1;
            }
        }

        // 默认 fp_rate 是 0.001，允许一定误差
        // 10000 次查询，期望误报约 10 次，允许最多 50 次（0.5%）
        assert!(
            false_positives < 50,
            "False positive rate too high: {}/10000",
            false_positives
        );
    }

    #[tokio::test]
    async fn test_concurrent_set_and_check() {
        use std::sync::Arc;

        let filter = Arc::new(BloomExistenceFilterPlugin::new().unwrap());
        let mut handles = vec![];

        // 并发设置 100 个 key
        for i in 0..100 {
            let f = Arc::clone(&filter);
            handles.push(tokio::spawn(async move {
                f.set(&format!("concurrent_key_{}", i)).await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // 验证所有 key 都存在
        for i in 0..100 {
            assert!(
                filter.check(&format!("concurrent_key_{}", i)).await,
                "Key {} should exist after concurrent set",
                i
            );
        }
    }
}
