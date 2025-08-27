use std::collections::HashMap;

use crate::storages::ShortLink;
use async_trait::async_trait;

pub struct BloomConfig {
    pub capacity: usize,
    pub fp_rate: f64,
}

/// 缓存查询结果
#[derive(Debug, Clone)]
pub enum CacheResult {
    /// 确定不存在
    NotFound,
    /// 存在但没有缓存值
    ExistsButNoValue,
    /// 成功获取到缓存值
    Found(ShortLink),
}

#[async_trait]
pub trait CompositeCacheTrait: Send + Sync {
    async fn get(&self, key: &str) -> CacheResult;
    async fn insert(&self, key: String, value: ShortLink);
    async fn remove(&self, key: &str);
    async fn invalidate_all(&self);

    /// 批量加载 Filter 和 Object Cache
    async fn load_cache(&self, links: HashMap<String, ShortLink>);

    /// 重新初始化 Filter
    async fn reconfigure(&self, config: BloomConfig);
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
    async fn clear(&self, count: usize, fp_rate: f64) {
        // 默认实现：子类可以选择覆盖
        tracing::debug!(
            "Not clearing Existence Filter, no operation defined. Count: {}, FP Rate: {}",
            count,
            fp_rate
        );
    }
}

#[async_trait]
pub trait ObjectCache: Send + Sync {
    async fn get(&self, key: &str) -> CacheResult;
    async fn insert(&self, key: String, value: ShortLink);
    async fn remove(&self, key: &str);
    async fn invalidate_all(&self);

    async fn load_object_cache(&self, _keys: HashMap<String, ShortLink>) {
        // 默认实现：子类可以选择覆盖
        tracing::debug!("Not loading Object Cache, no operation defined");
    }
}
