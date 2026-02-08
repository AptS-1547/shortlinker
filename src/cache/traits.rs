use std::collections::HashMap;

use crate::errors::Result;
use crate::storage::ShortLink;
use async_trait::async_trait;

pub struct BloomConfig {
    pub capacity: usize,
    pub fp_rate: f64,
}

/// 缓存健康检查状态
#[derive(Debug, Clone)]
pub struct CacheHealthStatus {
    pub status: String,
    pub cache_type: String,
    pub bloom_filter_enabled: bool,
    pub negative_cache_enabled: bool,
    pub error: Option<String>,
}

/// 缓存查询结果
#[derive(Debug, Clone)]
pub enum CacheResult {
    /// 确定不存在（来自 Negative Cache 或数据库确认）
    NotFound,
    /// 缓存未命中，需要回源数据库
    Miss,
    /// 成功获取到缓存值
    Found(ShortLink),
}

#[async_trait]
pub trait CompositeCacheTrait: Send + Sync {
    async fn get(&self, key: &str) -> CacheResult;
    async fn insert(&self, key: &str, value: ShortLink, ttl_secs: Option<u64>);
    async fn remove(&self, key: &str);
    /// 清空 Object Cache 和 Negative Cache。
    ///
    /// **注意**：不会清理 Bloom Filter。如果需要完整重置（包括 Bloom Filter 重建），
    /// 请使用 [`rebuild_all`](Self::rebuild_all)。
    async fn invalidate_all(&self);

    /// 完整重置所有缓存层，包括原子重建 Bloom Filter。
    ///
    /// 内部自行从数据库加载短码列表，然后：
    /// 1. 原子重建 Bloom Filter（无空窗期）
    /// 2. 清空 Object Cache
    /// 3. 清空 Negative Cache
    ///
    /// // BUG: 在 `load_all_codes()` 到 Bloom swap 之间的极窄窗口内，并发 `create_link`
    /// // 写入的 key 可能不在新 Bloom 中，导致该链接短暂返回 404（直到下次 reload）。
    /// // 窗口为毫秒级，reload 为低频操作，影响可忽略。
    async fn rebuild_all(&self) -> Result<()>;

    /// 标记 key 为不存在（写入 Negative Cache）
    async fn mark_not_found(&self, key: &str);

    /// 批量加载 Filter 和 Object Cache
    async fn load_cache(&self, links: HashMap<String, ShortLink>);

    /// 只加载 Bloom Filter（不加载 Object Cache）
    async fn load_bloom(&self, codes: &[String]);

    /// 重新初始化 Filter
    async fn reconfigure(&self, config: BloomConfig) -> Result<()>;

    /// 直接检查 Bloom Filter（不查 Object Cache / Negative Cache）
    /// - `false` = 一定不存在
    /// - `true` = 可能存在（有误报可能）
    async fn bloom_check(&self, key: &str) -> bool;

    /// 健康检查 - 返回缓存类型和状态
    async fn health_check(&self) -> CacheHealthStatus;
}

#[async_trait]
pub trait ExistenceFilter: Send + Sync {
    /// 在访问后端前先判断是否可能存在
    /// - `false` 表示**一定不存在**
    /// - `true` 表示**可能存在**
    async fn check(&self, key: &str) -> bool;

    /// 设置新值进入 Filter（例如将 key 加入 Bloom Filter）
    async fn set(&self, key: &str);

    /// 批量设置（用于从数据库或持久层导入）
    async fn bulk_set(&self, keys: &[String]);

    /// 清空整个 Filter（重载、重建场景）
    async fn clear(&self, count: usize, fp_rate: f64) -> Result<()> {
        // 默认实现：子类可以选择覆盖
        tracing::trace!(
            "Not clearing Existence Filter, no operation defined. Count: {}, FP Rate: {}",
            count,
            fp_rate
        );
        Ok(())
    }

    /// 用提供的 keys 原子重建 Filter。
    ///
    /// 默认实现为 `clear` + `bulk_set`（非原子）。
    /// Bloom Filter 实现会在锁外构建新实例后原子交换，消除空窗期。
    async fn rebuild(&self, keys: &[String], count: usize, fp_rate: f64) -> Result<()> {
        self.clear(count, fp_rate).await?;
        self.bulk_set(keys).await;
        Ok(())
    }
}

#[async_trait]
pub trait ObjectCache: Send + Sync {
    async fn get(&self, key: &str) -> CacheResult;
    async fn insert(&self, key: &str, value: ShortLink, ttl_secs: Option<u64>);
    async fn remove(&self, key: &str);
    async fn invalidate_all(&self);

    /// Returns the number of entries in the cache (for metrics)
    fn entry_count(&self) -> u64 {
        0 // Default: unknown/not supported
    }

    async fn load_object_cache(&self, _keys: HashMap<String, ShortLink>) {
        // 默认实现：子类可以选择覆盖
        tracing::trace!("Not loading Object Cache, no operation defined");
    }
}

/// 负向缓存：存储确认不存在的 key
#[async_trait]
pub trait NegativeCache: Send + Sync {
    /// 检查 key 是否在 Negative Cache 中（确认不存在）
    async fn contains(&self, key: &str) -> bool;

    /// 标记 key 为不存在
    async fn mark(&self, key: &str);

    /// 清除指定 key（用于新增数据时）
    async fn remove(&self, key: &str);

    /// 清空所有
    async fn clear(&self);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_link(code: &str) -> ShortLink {
        ShortLink {
            code: code.to_string(),
            target: "https://example.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
            password: None,
            click: 0,
        }
    }

    #[test]
    fn test_cache_result_not_found() {
        let result = CacheResult::NotFound;
        assert!(matches!(result, CacheResult::NotFound));

        // 测试 Debug trait
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("NotFound"));
    }

    #[test]
    fn test_cache_result_miss() {
        let result = CacheResult::Miss;
        assert!(matches!(result, CacheResult::Miss));

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("Miss"));
    }

    #[test]
    fn test_cache_result_found() {
        let link = create_test_link("test_code");
        let result = CacheResult::Found(link.clone());

        match result {
            CacheResult::Found(found_link) => {
                assert_eq!(found_link.code, "test_code");
                assert_eq!(found_link.target, "https://example.com");
            }
            _ => panic!("Expected Found variant"),
        }
    }

    #[test]
    fn test_cache_result_clone() {
        let link = create_test_link("clone_test");
        let result = CacheResult::Found(link);
        let cloned = result.clone();

        match cloned {
            CacheResult::Found(found_link) => {
                assert_eq!(found_link.code, "clone_test");
            }
            _ => panic!("Expected Found variant after clone"),
        }

        // NotFound 和 Miss 也应该可以 clone
        let not_found = CacheResult::NotFound;
        let _ = not_found.clone();

        let miss = CacheResult::Miss;
        let _ = miss.clone();
    }

    #[test]
    fn test_bloom_config() {
        let config = BloomConfig {
            capacity: 10000,
            fp_rate: 0.01,
        };

        assert_eq!(config.capacity, 10000);
        assert!((config.fp_rate - 0.01).abs() < f64::EPSILON);
    }
}
