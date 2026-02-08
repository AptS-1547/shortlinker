//! Analytics service layer
//!
//! Provides unified business logic for analytics queries, shared between
//! HTTP API, IPC, CLI and TUI interfaces.
//!
//! # Query Strategies
//!
//! - 原始查询（get_trends, get_top_links 等）：从 click_logs 表实时聚合，适用于小数据量
//! - v2 查询（get_trends_v2 等）：从汇总表读取，性能更好，适用于大数据量
//!
//! 调用方可根据数据规模选择使用哪套方法。

use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use sea_orm::{DbBackend, sea_query::Expr};
use tracing::{debug, info};

use crate::errors::ShortlinkerError;
use crate::storage::SeaOrmStorage;

use migration::entities::click_log;

// ============ 公共类型定义 ============

/// 分组方式
#[derive(Debug, Clone, Copy, Default)]
pub enum GroupBy {
    Hour,
    #[default]
    Day,
    Week,
    Month,
}

/// 点击趋势数据
#[derive(Debug, Clone)]
pub struct TrendData {
    pub labels: Vec<String>,
    pub values: Vec<u64>,
}

/// 热门链接
#[derive(Debug, Clone)]
pub struct TopLink {
    pub code: String,
    pub clicks: u64,
}

/// 来源统计
#[derive(Debug, Clone)]
pub struct ReferrerStats {
    pub referrer: String,
    pub count: u64,
    pub percentage: f64,
}

/// 地理位置统计
#[derive(Debug, Clone)]
pub struct GeoStats {
    pub country: String,
    pub city: Option<String>,
    pub count: u64,
}

/// 单链接分析数据
#[derive(Debug, Clone)]
pub struct LinkAnalytics {
    pub code: String,
    pub total_clicks: u64,
    pub trend: TrendData,
    pub top_referrers: Vec<ReferrerStats>,
    pub geo_distribution: Vec<GeoStats>,
}

/// 设备分析数据
#[derive(Debug, Clone)]
pub struct DeviceAnalytics {
    pub browsers: Vec<CategoryStats>,
    pub operating_systems: Vec<CategoryStats>,
    pub devices: Vec<CategoryStats>,
    pub bot_percentage: f64,
    pub total_with_ua: u64,
}

/// 分类统计
#[derive(Debug, Clone)]
pub struct CategoryStats {
    pub name: String,
    pub count: u64,
    pub percentage: f64,
}

// ============ AnalyticsService ============

/// Analytics 服务
pub struct AnalyticsService {
    storage: Arc<SeaOrmStorage>,
}

impl AnalyticsService {
    /// 创建 AnalyticsService 实例
    pub fn new(storage: Arc<SeaOrmStorage>) -> Self {
        Self { storage }
    }

    /// 解析日期范围，支持 RFC3339 和 YYYY-MM-DD 格式
    pub fn parse_date_range(
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> (DateTime<Utc>, DateTime<Utc>) {
        match (start_date, end_date) {
            (Some(s), Some(e)) => {
                let start = Self::parse_date(s).unwrap_or_else(|| Self::default_date_range().0);
                let end = Self::parse_date(e).unwrap_or_else(|| Self::default_date_range().1);
                (start, end)
            }
            _ => Self::default_date_range(),
        }
    }

    /// 严格解析日期范围，解析失败时返回错误
    ///
    /// 与 `parse_date_range` 不同，此方法不会静默回退到默认值
    pub fn parse_date_range_strict(
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<(DateTime<Utc>, DateTime<Utc>), ShortlinkerError> {
        match (start_date, end_date) {
            (Some(s), Some(e)) => {
                let start = Self::parse_date(s).ok_or_else(|| {
                    ShortlinkerError::analytics_invalid_date_range(format!(
                        "Invalid start date format: '{}'. Supported formats: RFC3339 or YYYY-MM-DD",
                        s
                    ))
                })?;
                let end = Self::parse_date(e).ok_or_else(|| {
                    ShortlinkerError::analytics_invalid_date_range(format!(
                        "Invalid end date format: '{}'. Supported formats: RFC3339 or YYYY-MM-DD",
                        e
                    ))
                })?;
                if start > end {
                    return Err(ShortlinkerError::analytics_invalid_date_range(
                        "Start date must not be later than end date",
                    ));
                }
                Ok((start, end))
            }
            (Some(_), None) => Err(ShortlinkerError::analytics_invalid_date_range(
                "Start date is provided but end date is missing",
            )),
            (None, Some(_)) => Err(ShortlinkerError::analytics_invalid_date_range(
                "End date is provided but start date is missing",
            )),
            (None, None) => Ok(Self::default_date_range()),
        }
    }

    fn parse_date(s: &str) -> Option<DateTime<Utc>> {
        DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|| {
                chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                    .ok()
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|dt| dt.and_utc())
            })
    }

    fn default_date_range() -> (DateTime<Utc>, DateTime<Utc>) {
        let end = Utc::now();
        let start = end - Duration::days(30);
        (start, end)
    }

    fn get_db_backend(&self) -> DbBackend {
        match self.storage.get_backend_name() {
            "sqlite" => DbBackend::Sqlite,
            "mysql" => DbBackend::MySql,
            _ => DbBackend::Postgres,
        }
    }

    fn date_format_expr(&self, group_by: GroupBy) -> Expr {
        let backend = self.get_db_backend();
        let (sqlite_fmt, mysql_fmt, pg_fmt) = match group_by {
            GroupBy::Hour => ("%Y-%m-%d %H:00", "%Y-%m-%d %H:00", "YYYY-MM-DD HH24:00"),
            GroupBy::Day => ("%Y-%m-%d", "%Y-%m-%d", "YYYY-MM-DD"),
            GroupBy::Week => ("%Y-W%W", "%Y-W%U", "IYYY-\"W\"IW"),
            GroupBy::Month => ("%Y-%m", "%Y-%m", "YYYY-MM"),
        };

        match backend {
            DbBackend::Sqlite => Expr::cust(format!("strftime('{}', clicked_at)", sqlite_fmt)),
            DbBackend::MySql => Expr::cust(format!("DATE_FORMAT(clicked_at, '{}')", mysql_fmt)),
            DbBackend::Postgres | _ => Expr::cust(format!("TO_CHAR(clicked_at, '{}')", pg_fmt)),
        }
    }

    /// 获取点击趋势
    pub async fn get_trends(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        group_by: GroupBy,
    ) -> Result<TrendData, ShortlinkerError> {
        info!(
            "Analytics: get_trends from {} to {}, group_by={:?}",
            start, end, group_by
        );

        let date_expr = self.date_format_expr(group_by);
        let results = self
            .storage
            .get_global_trend(start, end, date_expr)
            .await
            .map_err(|e| {
                ShortlinkerError::analytics_query_failed(format!("Trend query failed: {}", e))
            })?;

        let mut labels = Vec::with_capacity(results.len());
        let mut values = Vec::with_capacity(results.len());
        for row in results {
            labels.push(row.label);
            values.push(row.count as u64);
        }

        debug!(
            "Analytics: get_trends returned {} data points",
            labels.len()
        );

        Ok(TrendData { labels, values })
    }

    /// 获取热门链接
    pub async fn get_top_links(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<TopLink>, ShortlinkerError> {
        info!(
            "Analytics: get_top_links from {} to {}, limit={}",
            start, end, limit
        );

        let limit = limit.min(100) as u64;
        let results = self
            .storage
            .get_top_links(start, end, limit)
            .await
            .map_err(|e| {
                ShortlinkerError::analytics_query_failed(format!("Top links query failed: {}", e))
            })?;

        let top_links: Vec<TopLink> = results
            .into_iter()
            .map(|row| TopLink {
                code: row.short_code,
                clicks: row.count as u64,
            })
            .collect();

        debug!(
            "Analytics: get_top_links returned {} links",
            top_links.len()
        );

        Ok(top_links)
    }

    /// 获取来源统计
    ///
    /// 优化：从聚合结果计算总数，避免额外的 count 查询
    pub async fn get_referrers(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<ReferrerStats>, ShortlinkerError> {
        info!(
            "Analytics: get_referrers from {} to {}, limit={}",
            start, end, limit
        );

        let limit = limit.min(100) as u64;
        let results = self
            .storage
            .get_global_referrers(start, end, limit)
            .await
            .map_err(|e| {
                ShortlinkerError::analytics_query_failed(format!(
                    "Referrer stats query failed: {}",
                    e
                ))
            })?;

        // 从聚合结果计算总数（limit 内的总数）
        let total: u64 = results.iter().map(|r| r.count as u64).sum();

        let referrer_stats: Vec<ReferrerStats> = results
            .into_iter()
            .map(|row| {
                let referrer = row.referrer.unwrap_or_else(|| "(direct)".to_string());
                let count = row.count as u64;
                let percentage = if total > 0 {
                    (count as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                ReferrerStats {
                    referrer,
                    count,
                    percentage,
                }
            })
            .collect();

        debug!(
            "Analytics: get_referrers returned {} referrers",
            referrer_stats.len()
        );

        Ok(referrer_stats)
    }

    /// 获取地理位置分布
    pub async fn get_geo_stats(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<GeoStats>, ShortlinkerError> {
        info!(
            "Analytics: get_geo_stats from {} to {}, limit={}",
            start, end, limit
        );

        let limit = limit.min(100) as u64;
        let results = self
            .storage
            .get_global_geo(start, end, limit)
            .await
            .map_err(|e| {
                ShortlinkerError::analytics_query_failed(format!("Geolocation query failed: {}", e))
            })?;

        let geo_stats: Vec<GeoStats> = results
            .into_iter()
            .map(|row| GeoStats {
                country: row.country.unwrap_or_else(|| "Unknown".to_string()),
                city: row.city,
                count: row.count as u64,
            })
            .collect();

        debug!(
            "Analytics: get_geo_stats returned {} locations",
            geo_stats.len()
        );

        Ok(geo_stats)
    }

    /// 获取单链接详细统计
    ///
    /// 使用 `tokio::try_join!` 并发执行 4 个查询，减少响应时间
    pub async fn get_link_analytics(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<LinkAnalytics, ShortlinkerError> {
        info!(
            "Analytics: get_link_analytics for '{}' from {} to {}",
            code, start, end
        );

        let date_expr = self.date_format_expr(GroupBy::Day);

        // 并发执行 4 个 storage 层查询
        let (total_clicks, trend_rows, referrer_rows, geo_rows) = tokio::try_join!(
            self.storage.count_link_clicks(code, start, end),
            self.storage.get_link_trend(code, start, end, date_expr),
            self.storage.get_link_referrers(code, start, end, 10),
            self.storage.get_link_geo(code, start, end, 10),
        )
        .map_err(|e| ShortlinkerError::analytics_query_failed(e.to_string()))?;

        // 转换趋势数据
        let trend = TrendData {
            labels: trend_rows.iter().map(|r| r.label.clone()).collect(),
            values: trend_rows.iter().map(|r| r.count as u64).collect(),
        };

        // 转换来源统计结果
        let top_referrers: Vec<ReferrerStats> = referrer_rows
            .into_iter()
            .map(|row| {
                let referrer = row.referrer.unwrap_or_else(|| "(direct)".to_string());
                let count = row.count as u64;
                let percentage = if total_clicks > 0 {
                    (count as f64 / total_clicks as f64) * 100.0
                } else {
                    0.0
                };
                ReferrerStats {
                    referrer,
                    count,
                    percentage,
                }
            })
            .collect();

        // 转换地理分布结果
        let geo_distribution: Vec<GeoStats> = geo_rows
            .into_iter()
            .map(|row| GeoStats {
                country: row.country.unwrap_or_else(|| "Unknown".to_string()),
                city: row.city,
                count: row.count as u64,
            })
            .collect();

        debug!(
            "Analytics: get_link_analytics for '{}' returned {} clicks, {} trend points",
            code,
            total_clicks,
            trend.labels.len()
        );

        Ok(LinkAnalytics {
            code: code.to_string(),
            total_clicks,
            trend,
            top_referrers,
            geo_distribution,
        })
    }

    /// 获取单链接设备分析数据
    ///
    /// 并发查询指定链接的浏览器、操作系统、设备类型和 Bot 统计
    pub async fn get_link_device_analytics(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<DeviceAnalytics, ShortlinkerError> {
        info!(
            "Analytics: get_link_device_analytics for '{}' from {} to {}, limit={}",
            code, start, end, limit
        );

        let limit = limit.min(20) as u64;

        // 并发执行 4 个查询
        let (browsers, os, devices, bot_stats) = tokio::try_join!(
            self.storage.get_link_browser_stats(code, start, end, limit),
            self.storage.get_link_os_stats(code, start, end, limit),
            self.storage.get_link_device_stats(code, start, end, limit),
            self.storage.get_link_bot_stats(code, start, end),
        )
        .map_err(|e| ShortlinkerError::analytics_query_failed(e.to_string()))?;

        // 转换为 CategoryStats 并计算百分比
        let to_category = |rows: Vec<crate::storage::backend::UaStatsRow>| -> Vec<CategoryStats> {
            let total: u64 = rows.iter().map(|r| r.count as u64).sum();
            rows.into_iter()
                .map(|r| {
                    let count = r.count as u64;
                    CategoryStats {
                        name: r.field_value.unwrap_or_else(|| "Unknown".to_string()),
                        count,
                        percentage: if total > 0 {
                            (count as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        },
                    }
                })
                .collect()
        };

        let (bot_count, total_with_ua) = bot_stats;
        let bot_percentage = if total_with_ua > 0 {
            (bot_count as f64 / total_with_ua as f64) * 100.0
        } else {
            0.0
        };

        debug!(
            "Analytics: get_link_device_analytics for '{}' returned {} browsers, {} os, {} devices, bot={:.1}%",
            code,
            browsers.len(),
            os.len(),
            devices.len(),
            bot_percentage
        );

        Ok(DeviceAnalytics {
            browsers: to_category(browsers),
            operating_systems: to_category(os),
            devices: to_category(devices),
            bot_percentage,
            total_with_ua: total_with_ua as u64,
        })
    }

    /// 获取设备分析数据
    ///
    /// 并发查询浏览器、操作系统、设备类型和 Bot 统计，
    /// 返回各维度的分类统计及百分比。
    pub async fn get_device_analytics(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<DeviceAnalytics, ShortlinkerError> {
        info!(
            "Analytics: get_device_analytics from {} to {}, limit={}",
            start, end, limit
        );

        let limit = limit.min(20) as u64;

        // 并发执行 4 个查询
        let (browsers, os, devices, bot_stats) = tokio::try_join!(
            self.storage.get_browser_stats(start, end, limit),
            self.storage.get_os_stats(start, end, limit),
            self.storage.get_device_stats(start, end, limit),
            self.storage.get_bot_stats(start, end),
        )
        .map_err(|e| ShortlinkerError::analytics_query_failed(e.to_string()))?;

        // 转换为 CategoryStats 并计算百分比
        let to_category = |rows: Vec<crate::storage::backend::UaStatsRow>| -> Vec<CategoryStats> {
            let total: u64 = rows.iter().map(|r| r.count as u64).sum();
            rows.into_iter()
                .map(|r| {
                    let count = r.count as u64;
                    CategoryStats {
                        name: r.field_value.unwrap_or_else(|| "Unknown".to_string()),
                        count,
                        percentage: if total > 0 {
                            (count as f64 / total as f64) * 100.0
                        } else {
                            0.0
                        },
                    }
                })
                .collect()
        };

        let (bot_count, total_with_ua) = bot_stats;
        let bot_percentage = if total_with_ua > 0 {
            (bot_count as f64 / total_with_ua as f64) * 100.0
        } else {
            0.0
        };

        debug!(
            "Analytics: get_device_analytics returned {} browsers, {} os, {} devices, bot={:.1}%",
            browsers.len(),
            os.len(),
            devices.len(),
            bot_percentage
        );

        Ok(DeviceAnalytics {
            browsers: to_category(browsers),
            operating_systems: to_category(os),
            devices: to_category(devices),
            bot_percentage,
            total_with_ua: total_with_ua as u64,
        })
    }

    /// 导出点击日志（带分页限制）
    pub async fn export_click_logs(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> Result<Vec<click_log::Model>, ShortlinkerError> {
        info!(
            "Analytics: export_click_logs from {} to {}, limit={}",
            start, end, limit
        );

        let limit = limit.min(100000);
        let logs = self
            .storage
            .export_click_logs(start, end, limit)
            .await
            .map_err(|e| {
                ShortlinkerError::analytics_query_failed(format!(
                    "Failed to export click logs: {}",
                    e
                ))
            })?;

        debug!(
            "Analytics: export_click_logs returned {} records",
            logs.len()
        );

        Ok(logs)
    }

    // ============ v2 查询方法（从汇总表读取） ============

    /// 获取点击趋势（从汇总表）
    ///
    /// 性能优于 get_trends，但需要汇总表有数据
    pub async fn get_trends_v2(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        group_by: GroupBy,
    ) -> Result<TrendData, ShortlinkerError> {
        info!(
            "Analytics: get_trends_v2 from {} to {}, group_by={:?}",
            start, end, group_by
        );

        let results = match group_by {
            GroupBy::Hour => self.storage.get_global_trend_from_hourly(start, end).await,
            GroupBy::Day => {
                let start_date = start.date_naive();
                let end_date = end.date_naive();
                self.storage
                    .get_global_trend_from_daily(start_date, end_date)
                    .await
            }
            GroupBy::Week | GroupBy::Month => {
                // 从天汇总读取，在 service 层做二次聚合
                let start_date = start.date_naive();
                let end_date = end.date_naive();
                let daily_results = self
                    .storage
                    .get_global_trend_from_daily(start_date, end_date)
                    .await
                    .map_err(|e| {
                        ShortlinkerError::analytics_query_failed(format!(
                            "Daily trend query failed: {}",
                            e
                        ))
                    })?;

                let format_str = match group_by {
                    GroupBy::Week => "%G-W%V",
                    GroupBy::Month => "%Y-%m",
                    _ => unreachable!(),
                };

                use std::collections::BTreeMap;
                let mut grouped: BTreeMap<String, i64> = BTreeMap::new();
                for row in &daily_results {
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(&row.label, "%Y-%m-%d") {
                        let key = date.format(format_str).to_string();
                        *grouped.entry(key).or_insert(0) += row.count;
                    }
                }

                Ok(grouped
                    .into_iter()
                    .map(|(label, count)| {
                        crate::storage::backend::TrendRow { label, count }
                    })
                    .collect())
            }
        }
        .map_err(|e| {
            ShortlinkerError::analytics_query_failed(format!("Trend query failed: {}", e))
        })?;

        let mut labels = Vec::with_capacity(results.len());
        let mut values = Vec::with_capacity(results.len());
        for row in results {
            labels.push(row.label);
            values.push(row.count as u64);
        }

        debug!(
            "Analytics: get_trends_v2 returned {} data points",
            labels.len()
        );

        Ok(TrendData { labels, values })
    }

    /// 获取单链接趋势（从汇总表）
    pub async fn get_link_trends_v2(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        group_by: GroupBy,
    ) -> Result<TrendData, ShortlinkerError> {
        info!(
            "Analytics: get_link_trends_v2 for '{}' from {} to {}, group_by={:?}",
            code, start, end, group_by
        );

        let results = match group_by {
            GroupBy::Hour => {
                self.storage
                    .get_link_trend_from_hourly(code, start, end)
                    .await
            }
            _ => {
                let start_date = start.date_naive();
                let end_date = end.date_naive();
                self.storage
                    .get_link_trend_from_daily(code, start_date, end_date)
                    .await
            }
        }
        .map_err(|e| {
            ShortlinkerError::analytics_query_failed(format!("Link trend query failed: {}", e))
        })?;

        let mut labels = Vec::with_capacity(results.len());
        let mut values = Vec::with_capacity(results.len());
        for row in results {
            labels.push(row.label);
            values.push(row.count as u64);
        }

        debug!(
            "Analytics: get_link_trends_v2 returned {} data points",
            labels.len()
        );

        Ok(TrendData { labels, values })
    }

    /// 获取来源统计（从汇总表）
    pub async fn get_link_referrers_v2(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<ReferrerStats>, ShortlinkerError> {
        info!(
            "Analytics: get_link_referrers_v2 for '{}' from {} to {}, limit={}",
            code, start, end, limit
        );

        let limit = limit.min(100) as usize;
        let results = self
            .storage
            .get_link_referrers_from_rollup(code, start, end, limit)
            .await
            .map_err(|e| {
                ShortlinkerError::analytics_query_failed(format!(
                    "Referrer stats query failed: {}",
                    e
                ))
            })?;

        let total: u64 = results.iter().map(|r| r.count as u64).sum();

        let referrer_stats: Vec<ReferrerStats> = results
            .into_iter()
            .map(|row| {
                let referrer = row.referrer.unwrap_or_else(|| "(direct)".to_string());
                let count = row.count as u64;
                let percentage = if total > 0 {
                    (count as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                ReferrerStats {
                    referrer,
                    count,
                    percentage,
                }
            })
            .collect();

        debug!(
            "Analytics: get_link_referrers_v2 returned {} referrers",
            referrer_stats.len()
        );

        Ok(referrer_stats)
    }

    /// 获取地理位置分布（从汇总表）
    pub async fn get_link_geo_v2(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<GeoStats>, ShortlinkerError> {
        info!(
            "Analytics: get_link_geo_v2 for '{}' from {} to {}, limit={}",
            code, start, end, limit
        );

        let limit = limit.min(100) as usize;
        let results = self
            .storage
            .get_link_geo_from_rollup(code, start, end, limit)
            .await
            .map_err(|e| {
                ShortlinkerError::analytics_query_failed(format!("Geolocation query failed: {}", e))
            })?;

        let geo_stats: Vec<GeoStats> = results
            .into_iter()
            .map(|row| GeoStats {
                country: row.country.unwrap_or_else(|| "Unknown".to_string()),
                city: row.city,
                count: row.count as u64,
            })
            .collect();

        debug!(
            "Analytics: get_link_geo_v2 returned {} locations",
            geo_stats.len()
        );

        Ok(geo_stats)
    }

    /// 获取热门链接（从汇总表）
    pub async fn get_top_links_v2(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u32,
    ) -> Result<Vec<TopLink>, ShortlinkerError> {
        info!(
            "Analytics: get_top_links_v2 from {} to {}, limit={}",
            start, end, limit
        );

        let start_date = start.date_naive();
        let end_date = end.date_naive();
        let limit = limit.min(100) as usize;

        let results = self
            .storage
            .get_top_links_from_daily(start_date, end_date, limit)
            .await
            .map_err(|e| {
                ShortlinkerError::analytics_query_failed(format!("Top links query failed: {}", e))
            })?;

        let top_links: Vec<TopLink> = results
            .into_iter()
            .map(|row| TopLink {
                code: row.short_code,
                clicks: row.count as u64,
            })
            .collect();

        debug!(
            "Analytics: get_top_links_v2 returned {} links",
            top_links.len()
        );

        Ok(top_links)
    }
}
