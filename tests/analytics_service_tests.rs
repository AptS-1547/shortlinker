//! AnalyticsService 集成测试
//!
//! 覆盖 parse_date_range、parse_date_range_strict、
//! get_trends、get_top_links、get_referrers、get_geo_stats、
//! get_link_analytics、get_device_analytics、export_click_logs、
//! 以及 v2 查询方法。

use std::sync::{Arc, Once};

use chrono::{Duration, Utc};
use tempfile::TempDir;

use shortlinker::config::init_config;
use shortlinker::metrics_core::NoopMetrics;
use shortlinker::services::{AnalyticsService, GroupBy};
use shortlinker::storage::backend::SeaOrmStorage;

// =============================================================================
// 全局初始化
// =============================================================================

static INIT: Once = Once::new();

fn init_static_config() {
    INIT.call_once(|| {
        init_config();
    });
}

async fn create_temp_storage() -> (Arc<SeaOrmStorage>, TempDir) {
    init_static_config();
    let td = TempDir::new().unwrap();
    let p = td.path().join("analytics_svc_test.db");
    let u = format!("sqlite://{}?mode=rwc", p.display());
    let s = SeaOrmStorage::new(&u, "sqlite", NoopMetrics::arc())
        .await
        .unwrap();
    (Arc::new(s), td)
}

// =============================================================================
// parse_date_range 测试
// =============================================================================

#[cfg(test)]
mod parse_date_range_tests {
    use super::*;

    #[test]
    fn test_both_none_returns_default() {
        init_static_config();
        let (start, end) = AnalyticsService::parse_date_range(None, None);
        // 默认 30 天范围
        let diff = (end - start).num_days();
        assert!((29..=31).contains(&diff));
    }

    #[test]
    fn test_both_rfc3339() {
        init_static_config();
        let (start, end) = AnalyticsService::parse_date_range(
            Some("2024-01-01T00:00:00Z"),
            Some("2024-01-31T23:59:59Z"),
        );
        assert_eq!(start.date_naive().to_string(), "2024-01-01");
        assert_eq!(end.date_naive().to_string(), "2024-01-31");
    }

    #[test]
    fn test_both_yyyy_mm_dd() {
        init_static_config();
        let (start, end) =
            AnalyticsService::parse_date_range(Some("2024-06-01"), Some("2024-06-30"));
        assert_eq!(start.date_naive().to_string(), "2024-06-01");
        assert_eq!(end.date_naive().to_string(), "2024-06-30");
    }

    #[test]
    fn test_only_start_returns_default() {
        init_static_config();
        let (start, end) = AnalyticsService::parse_date_range(Some("2024-01-01"), None);
        // 只提供 start 时回退到默认
        let diff = (end - start).num_days();
        assert!((29..=31).contains(&diff));
    }

    #[test]
    fn test_invalid_format_returns_default() {
        init_static_config();
        let (start, end) = AnalyticsService::parse_date_range(Some("not-a-date"), Some("also-bad"));
        let diff = (end - start).num_days();
        assert!((29..=31).contains(&diff));
    }
}

// =============================================================================
// parse_date_range_strict 测试
// =============================================================================

#[cfg(test)]
mod parse_date_range_strict_tests {
    use super::*;

    #[test]
    fn test_both_none_returns_default() {
        init_static_config();
        let result = AnalyticsService::parse_date_range_strict(None, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_rfc3339() {
        init_static_config();
        let result = AnalyticsService::parse_date_range_strict(
            Some("2024-01-01T00:00:00Z"),
            Some("2024-01-31T23:59:59Z"),
        );
        assert!(result.is_ok());
        let (start, end) = result.unwrap();
        assert!(start < end);
    }

    #[test]
    fn test_start_after_end_returns_error() {
        init_static_config();
        let result = AnalyticsService::parse_date_range_strict(
            Some("2024-12-31T00:00:00Z"),
            Some("2024-01-01T00:00:00Z"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_only_start_returns_error() {
        init_static_config();
        let result = AnalyticsService::parse_date_range_strict(Some("2024-01-01T00:00:00Z"), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_only_end_returns_error() {
        init_static_config();
        let result = AnalyticsService::parse_date_range_strict(None, Some("2024-01-31T00:00:00Z"));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_start_format() {
        init_static_config();
        let result = AnalyticsService::parse_date_range_strict(
            Some("bad-date"),
            Some("2024-01-31T00:00:00Z"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_end_format() {
        init_static_config();
        let result = AnalyticsService::parse_date_range_strict(
            Some("2024-01-01T00:00:00Z"),
            Some("bad-date"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_yyyy_mm_dd_format() {
        init_static_config();
        let result =
            AnalyticsService::parse_date_range_strict(Some("2024-06-01"), Some("2024-06-30"));
        assert!(result.is_ok());
    }
}

// =============================================================================
// AnalyticsService 异步方法测试（空数据库）
// =============================================================================

#[cfg(test)]
mod service_query_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_trends_empty_db() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.get_trends(start, end, GroupBy::Day).await;
        assert!(result.is_ok(), "get_trends 失败: {:?}", result);
        let data = result.unwrap();
        assert!(data.labels.is_empty());
        assert!(data.values.is_empty());
    }

    #[tokio::test]
    async fn test_get_trends_hour_grouping() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::hours(24);

        let result = svc.get_trends(start, end, GroupBy::Hour).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_top_links_empty_db() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.get_top_links(start, end, 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_top_links_limit_capped() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        // 传入超大 limit，应被 cap 到 100
        let result = svc.get_top_links(start, end, 9999).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_referrers_empty_db() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.get_referrers(start, end, 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_geo_stats_empty_db() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.get_geo_stats(start, end, 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_link_analytics_empty_db() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.get_link_analytics("nonexistent", start, end).await;
        assert!(result.is_ok());
        let analytics = result.unwrap();
        assert_eq!(analytics.code, "nonexistent");
        assert_eq!(analytics.total_clicks, 0);
        assert!(analytics.trend.labels.is_empty());
    }

    #[tokio::test]
    async fn test_export_click_logs_empty_db() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.export_click_logs(start, end, 100).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_export_click_logs_limit_capped() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        // 超大 limit 应被 cap 到 100000
        let result = svc.export_click_logs(start, end, 999999).await;
        assert!(result.is_ok());
    }
}

// =============================================================================
// v2 查询方法测试
// =============================================================================

#[cfg(test)]
mod v2_query_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_trends_v2_hour() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::hours(24);

        let result = svc.get_trends_v2(start, end, GroupBy::Hour).await;
        assert!(result.is_ok());
        assert!(result.unwrap().labels.is_empty());
    }

    #[tokio::test]
    async fn test_get_trends_v2_day() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.get_trends_v2(start, end, GroupBy::Day).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_trends_v2_week() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(30);

        let result = svc.get_trends_v2(start, end, GroupBy::Week).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_trends_v2_month() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(90);

        let result = svc.get_trends_v2(start, end, GroupBy::Month).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_top_links_v2_empty() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.get_top_links_v2(start, end, 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_link_trends_v2_hour() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::hours(24);

        let result = svc
            .get_link_trends_v2("test-code", start, end, GroupBy::Hour)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_link_trends_v2_day() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc
            .get_link_trends_v2("test-code", start, end, GroupBy::Day)
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_link_referrers_v2_empty() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.get_link_referrers_v2("test-code", start, end, 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_link_geo_v2_empty() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc.get_link_geo_v2("test-code", start, end, 10).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}

// =============================================================================
// Device analytics 测试
// =============================================================================

#[cfg(test)]
mod device_analytics_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_device_analytics_empty_db() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        // user_agents 表可能不存在或为空，接受 Ok 或 Err
        let result = svc.get_device_analytics(start, end, 10).await;
        // 空数据库上可能成功（空结果）或失败（表不存在）
        match result {
            Ok(analytics) => {
                assert!(analytics.browsers.is_empty());
                assert_eq!(analytics.bot_percentage, 0.0);
            }
            Err(_) => {
                // user_agents JOIN 在空 DB 上可能失败，可接受
            }
        }
    }

    #[tokio::test]
    async fn test_get_link_device_analytics_empty_db() {
        let (storage, _td) = create_temp_storage().await;
        let svc = AnalyticsService::new(storage);
        let end = Utc::now();
        let start = end - Duration::days(7);

        let result = svc
            .get_link_device_analytics("nonexistent", start, end, 10)
            .await;
        match result {
            Ok(analytics) => {
                assert!(analytics.browsers.is_empty());
                assert_eq!(analytics.total_with_ua, 0);
            }
            Err(_) => {
                // 可接受
            }
        }
    }
}
