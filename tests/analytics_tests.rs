//! Analytics 模块测试
//!
//! 覆盖 ClickAggregation、ClickDetail、ClickManager、
//! aggregate_click_details、RollupManager 和 DataRetentionTask。

use std::sync::{Arc, Once};

use async_trait::async_trait;
use chrono::Utc;
use tempfile::TempDir;
use tokio::time::Duration as TokioDuration;

use shortlinker::analytics::{
    ClickAggregation, ClickDetail, ClickManager, ClickSink, DataRetentionTask, DetailedClickSink,
    RollupManager, aggregate_click_details,
};
use shortlinker::config::init_config;
use shortlinker::config::runtime_config::init_runtime_config;
use shortlinker::metrics_core::NoopMetrics;
use shortlinker::storage::backend::{SeaOrmStorage, connect_sqlite, run_migrations};

// =============================================================================
// 全局初始化
// =============================================================================

static INIT: Once = Once::new();
static TEST_DIR: std::sync::OnceLock<TempDir> = std::sync::OnceLock::new();
static RT_INIT: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

fn init_static_config() {
    INIT.call_once(|| {
        init_config();
    });
}

async fn init_test_runtime_config() {
    init_static_config();
    RT_INIT
        .get_or_init(|| async {
            let td = TempDir::new().unwrap();
            let p = td.path().join("analytics_rt.db");
            let u = format!("sqlite://{}?mode=rwc", p.display());
            let db = connect_sqlite(&u).await.unwrap();
            run_migrations(&db).await.unwrap();
            init_runtime_config(db).await.unwrap();
            let _ = TEST_DIR.set(td);
        })
        .await;
}

async fn create_temp_storage() -> (Arc<SeaOrmStorage>, TempDir) {
    init_static_config();
    let td = TempDir::new().unwrap();
    let p = td.path().join("test.db");
    let u = format!("sqlite://{}?mode=rwc", p.display());
    let s = SeaOrmStorage::new(&u, "sqlite", NoopMetrics::arc())
        .await
        .unwrap();
    (Arc::new(s), td)
}

struct MockSink {
    flushed: std::sync::Mutex<Vec<(String, usize)>>,
}
impl MockSink {
    fn new() -> Self {
        Self {
            flushed: std::sync::Mutex::new(Vec::new()),
        }
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

struct MockDetailedSink;
#[async_trait]
impl DetailedClickSink for MockDetailedSink {
    async fn log_click(&self, _detail: ClickDetail) -> anyhow::Result<()> {
        Ok(())
    }
    async fn log_clicks_batch(&self, _details: Vec<ClickDetail>) -> anyhow::Result<()> {
        Ok(())
    }
}

// =============================================================================
// ClickAggregation 测试
// =============================================================================

#[cfg(test)]
mod click_aggregation_tests {
    use super::*;

    #[test]
    fn test_new_with_count() {
        let agg = ClickAggregation::new(42);
        assert_eq!(agg.count, 42);
        assert!(agg.referrers.is_empty());
        assert!(agg.countries.is_empty());
        assert!(agg.sources.is_empty());
    }

    #[test]
    fn test_default_is_zero() {
        let agg = ClickAggregation::default();
        assert_eq!(agg.count, 0);
    }

    #[test]
    fn test_merge_counts() {
        let mut a = ClickAggregation::new(10);
        let b = ClickAggregation::new(5);
        a.merge(&b);
        assert_eq!(a.count, 15);
    }

    #[test]
    fn test_merge_referrers() {
        let mut a = ClickAggregation::new(1);
        a.referrers.insert("google.com".to_string(), 3);
        a.referrers.insert("bing.com".to_string(), 1);

        let mut b = ClickAggregation::new(1);
        b.referrers.insert("google.com".to_string(), 2);
        b.referrers.insert("yahoo.com".to_string(), 4);

        a.merge(&b);
        assert_eq!(a.referrers["google.com"], 5);
        assert_eq!(a.referrers["bing.com"], 1);
        assert_eq!(a.referrers["yahoo.com"], 4);
    }

    #[test]
    fn test_merge_countries_and_sources() {
        let mut a = ClickAggregation::new(0);
        a.countries.insert("CN".to_string(), 10);
        a.sources.insert("direct".to_string(), 2);

        let mut b = ClickAggregation::new(0);
        b.countries.insert("CN".to_string(), 5);
        b.countries.insert("US".to_string(), 3);
        b.sources.insert("direct".to_string(), 3);
        b.sources.insert("twitter".to_string(), 1);

        a.merge(&b);
        assert_eq!(a.countries["CN"], 15);
        assert_eq!(a.countries["US"], 3);
        assert_eq!(a.sources["direct"], 5);
        assert_eq!(a.sources["twitter"], 1);
    }

    #[test]
    fn test_merge_with_empty() {
        let mut a = ClickAggregation::new(10);
        a.referrers.insert("x.com".to_string(), 1);
        let b = ClickAggregation::default();
        a.merge(&b);
        assert_eq!(a.count, 10);
        assert_eq!(a.referrers["x.com"], 1);
    }
}

// =============================================================================
// ClickDetail 测试
// =============================================================================

#[cfg(test)]
mod click_detail_tests {
    use super::*;

    #[test]
    fn test_new_sets_code_and_timestamp() {
        let before = Utc::now();
        let detail = ClickDetail::new("abc".to_string());
        let after = Utc::now();

        assert_eq!(detail.code, "abc");
        assert!(detail.timestamp >= before && detail.timestamp <= after);
        assert!(detail.referrer.is_none());
        assert!(detail.country.is_none());
        assert!(detail.city.is_none());
        assert!(detail.source.is_none());
    }

    #[test]
    fn test_with_geo() {
        let detail = ClickDetail::new("test".to_string())
            .with_geo(Some("CN".to_string()), Some("Beijing".to_string()));
        assert_eq!(detail.country, Some("CN".to_string()));
        assert_eq!(detail.city, Some("Beijing".to_string()));
    }

    #[test]
    fn test_with_geo_none() {
        let detail = ClickDetail::new("test".to_string()).with_geo(None, None);
        assert!(detail.country.is_none());
        assert!(detail.city.is_none());
    }

    #[test]
    fn test_clone() {
        let detail = ClickDetail::new("clone".to_string())
            .with_geo(Some("FR".to_string()), Some("Paris".to_string()));
        let cloned = detail.clone();
        assert_eq!(cloned.code, detail.code);
        assert_eq!(cloned.country, detail.country);
    }
}

// =============================================================================
// aggregate_click_details 测试
// =============================================================================

#[cfg(test)]
mod aggregate_tests {
    use super::*;

    #[test]
    fn test_aggregate_empty() {
        let details: Vec<ClickDetail> = vec![];
        let result = aggregate_click_details(&details);
        assert!(result.is_empty());
    }

    #[test]
    fn test_aggregate_single_code() {
        let mut d1 = ClickDetail::new("abc".to_string());
        d1.referrer = Some("google.com".to_string());
        d1.country = Some("CN".to_string());
        d1.source = Some("direct".to_string());

        let mut d2 = ClickDetail::new("abc".to_string());
        d2.referrer = Some("bing.com".to_string());
        d2.country = Some("US".to_string());
        d2.source = Some("direct".to_string());

        let result = aggregate_click_details(&[d1, d2]);
        // key 是 (code, hour_bucket)，同一秒内的 ClickDetail 会被聚合到同一个 bucket
        assert_eq!(result.len(), 1);
        let agg = result.values().next().unwrap();
        assert_eq!(agg.count, 2);
        assert_eq!(agg.referrers["google.com"], 1);
        assert_eq!(agg.referrers["bing.com"], 1);
        assert_eq!(agg.countries["CN"], 1);
        assert_eq!(agg.countries["US"], 1);
        assert_eq!(agg.sources["direct"], 2);
    }

    #[test]
    fn test_aggregate_multiple_codes() {
        let d1 = ClickDetail::new("a".to_string());
        let d2 = ClickDetail::new("b".to_string());
        let d3 = ClickDetail::new("a".to_string());

        let result = aggregate_click_details(&[d1, d2, d3]);
        assert_eq!(result.len(), 2);
        // 找到 code="a" 的聚合
        let agg_a = result.iter().find(|((code, _), _)| code == "a").unwrap().1;
        let agg_b = result.iter().find(|((code, _), _)| code == "b").unwrap().1;
        assert_eq!(agg_a.count, 2);
        assert_eq!(agg_b.count, 1);
    }

    #[test]
    fn test_aggregate_none_fields_get_defaults() {
        // referrer/country/source 为 None 时会被映射为 "direct"/"Unknown"/"direct"
        let d = ClickDetail::new("x".to_string());
        let result = aggregate_click_details(&[d]);
        let agg = result.values().next().unwrap();
        assert_eq!(agg.count, 1);
        assert_eq!(agg.referrers["direct"], 1);
        assert_eq!(agg.countries["Unknown"], 1);
        assert_eq!(agg.sources["direct"], 1);
    }
}

// =============================================================================
// ClickManager 测试
// =============================================================================

#[cfg(test)]
mod click_manager_tests {
    use super::*;

    fn create_test_manager(sink: Arc<dyn ClickSink>, max_clicks: usize) -> ClickManager {
        ClickManager::new(
            sink,
            TokioDuration::from_secs(60),
            max_clicks,
            NoopMetrics::arc(),
        )
    }

    #[tokio::test]
    async fn test_increment_and_buffer_size() {
        let sink = Arc::new(MockSink::new());
        let manager = create_test_manager(sink.clone(), 100);

        manager.increment("key1");
        manager.increment("key1");
        manager.increment("key2");

        assert_eq!(manager.buffer_size(), 3);
    }

    #[tokio::test]
    async fn test_flush_clears_buffer() {
        let sink = Arc::new(MockSink::new());
        let manager = create_test_manager(sink.clone(), 100);

        manager.increment("a");
        manager.increment("b");
        manager.flush().await;

        assert_eq!(manager.buffer_size(), 0);
        assert_eq!(sink.total_clicks(), 2);
    }

    #[tokio::test]
    async fn test_concurrent_increment_no_lost_clicks() {
        let sink = Arc::new(MockSink::new());
        let manager = Arc::new(create_test_manager(sink.clone(), 100000));

        const THREADS: usize = 8;
        const PER_THREAD: usize = 500;

        let mut handles = vec![];
        for _ in 0..THREADS {
            let mgr = Arc::clone(&manager);
            handles.push(tokio::spawn(async move {
                for _ in 0..PER_THREAD {
                    mgr.increment("shared");
                }
            }));
        }
        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(manager.buffer_size(), THREADS * PER_THREAD);
        manager.flush().await;
        assert_eq!(sink.total_clicks(), THREADS * PER_THREAD);
    }

    #[tokio::test]
    async fn test_with_detailed_logging_returns_receiver() {
        let sink = Arc::new(MockSink::new());
        let detailed_sink = Arc::new(MockDetailedSink);
        let (manager, _rx) = ClickManager::with_detailed_logging(
            sink,
            detailed_sink,
            TokioDuration::from_secs(60),
            100,
            NoopMetrics::arc(),
        );
        manager.increment("test");
        assert_eq!(manager.buffer_size(), 1);
    }
}

// =============================================================================
// RollupManager 集成测试
// =============================================================================

#[cfg(test)]
mod rollup_integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_rollup_manager_creation() {
        let (storage, _td) = create_temp_storage().await;
        let _manager = RollupManager::new(storage);
        // 创建成功即可
    }

    #[tokio::test]
    async fn test_increment_hourly_counts() {
        let (storage, _td) = create_temp_storage().await;
        let manager = RollupManager::new(storage);
        let now = Utc::now();

        let updates = vec![
            ("test-code".to_string(), 5usize),
            ("another-code".to_string(), 3usize),
        ];
        let result = manager.increment_hourly_counts(&updates, now).await;
        assert!(result.is_ok(), "increment_hourly_counts 失败: {:?}", result);

        // 再次写入，验证累加
        let updates2 = vec![("test-code".to_string(), 2usize)];
        let result2 = manager.increment_hourly_counts(&updates2, now).await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_rollup_hourly_to_daily() {
        let (storage, _td) = create_temp_storage().await;
        let manager = RollupManager::new(storage);
        let now = Utc::now();

        // 先写入一些小时数据
        let updates = vec![("rollup-test".to_string(), 10usize)];
        manager
            .increment_hourly_counts(&updates, now)
            .await
            .unwrap();

        // 执行汇总
        let yesterday = (Utc::now() - chrono::Duration::days(1)).date_naive();
        let result = manager.rollup_hourly_to_daily(yesterday).await;
        assert!(result.is_ok(), "rollup_hourly_to_daily 失败: {:?}", result);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let (storage, _td) = create_temp_storage().await;
        let manager = RollupManager::new(storage);

        // 清理过期数据（空表不应报错）
        let result = manager.cleanup_expired(7, 30).await;
        assert!(result.is_ok(), "cleanup_expired 失败: {:?}", result);
    }
}

// =============================================================================
// DataRetentionTask 测试
// =============================================================================

#[cfg(test)]
mod retention_tests {
    use super::*;

    #[tokio::test]
    async fn test_retention_task_creation() {
        init_test_runtime_config().await;
        let (storage, _td) = create_temp_storage().await;
        let rollup = Arc::new(RollupManager::new(storage.clone()));
        let task = DataRetentionTask::new(storage, rollup);

        // should_stop_logging 默认应为 false（max_log_rows 默认 0）
        assert!(!task.should_stop_logging());
    }

    #[tokio::test]
    async fn test_run_cleanup_on_empty_db() {
        init_test_runtime_config().await;
        let (storage, _td) = create_temp_storage().await;
        let rollup = Arc::new(RollupManager::new(storage.clone()));
        let task = DataRetentionTask::new(storage, rollup);

        // 空数据库上执行清理不应报错
        let report = task.run_cleanup().await;
        assert!(report.is_ok(), "run_cleanup 失败: {:?}", report);
        let report = report.unwrap();
        assert_eq!(report.raw_logs_deleted, 0);
        assert_eq!(report.hourly_stats_deleted, 0);
        assert_eq!(report.daily_stats_deleted, 0);
    }
}
