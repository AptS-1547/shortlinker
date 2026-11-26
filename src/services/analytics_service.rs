//! Analytics service layer
//!
//! Provides unified business logic for analytics queries, shared between
//! HTTP API, IPC and CLI interfaces.
//!
//! # Query Strategies
//!
//! - 原始查询（get_trends, get_top_links 等）：从 click_logs 表实时聚合，适用于小数据量
//! - v2 查询（get_trends_v2 等）：从汇总表读取，性能更好，适用于大数据量
//!
//! 调用方可根据数据规模选择使用哪套方法。

use std::pin::Pin;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};
use futures_util::{Stream, StreamExt};
use sea_orm::{DbBackend, sea_query::Expr};
use tracing::{debug, info};

use crate::errors::ShortlinkerError;
use crate::storage::SeaOrmStorage;
use crate::utils::percentage;

// ============ 公共类型定义 ============

/// 分组方式
#[derive(Debug, Clone, Copy, Default)]
pub enum GroupBy {
    /// 按小时分组，标签形如 `2026-01-01 08:00`（仅原始查询支持）
    Hour,
    /// 按天分组（默认值），标签形如 `2026-01-01`
    #[default]
    Day,
    /// 按周分组，标签形如 `2026-W02`
    Week,
    /// 按月分组，标签形如 `2026-01`
    Month,
}

/// 点击趋势数据
#[derive(Debug, Clone)]
pub struct TrendData {
    /// 各时间桶的标签，格式由 `GroupBy` 决定，与 `values` 一一对应
    pub labels: Vec<String>,
    /// 各时间桶内的点击次数，与 `labels` 一一对应
    pub values: Vec<u64>,
}

/// 热门链接
#[derive(Debug, Clone)]
pub struct TopLink {
    /// 短码
    pub code: String,
    /// 统计区间内的点击次数
    pub clicks: u64,
}

/// 来源统计
#[derive(Debug, Clone)]
pub struct ReferrerStats {
    /// 来源页 URL 或主机；直接访问（无 Referer）时为 `(direct)`
    pub referrer: String,
    /// 该来源的点击次数
    pub count: u64,
    /// 占本次返回的各来源点击数之和的百分比（0-100）
    pub percentage: f64,
}

/// 地理位置统计
#[derive(Debug, Clone)]
pub struct GeoStats {
    /// 国家/地区名称，来自 GeoIP 解析；无法解析时为 `Unknown`
    pub country: String,
    /// 城市名称；GeoIP 未命中城市信息时为 `None`
    pub city: Option<String>,
    /// 该地区的点击次数
    pub count: u64,
}

/// 单链接分析数据
#[derive(Debug, Clone)]
pub struct LinkAnalytics {
    /// 短码
    pub code: String,
    /// 统计区间内的点击总数
    pub total_clicks: u64,
    /// 区间内按天分组的点击趋势
    pub trend: TrendData,
    /// 点击次数最高的前 10 个来源
    pub top_referrers: Vec<ReferrerStats>,
    /// 点击次数最高的前 10 个地区分布
    pub geo_distribution: Vec<GeoStats>,
}

/// 设备分析数据
#[derive(Debug, Clone)]
pub struct DeviceAnalytics {
    /// 浏览器维度统计，按点击次数降序
    pub browsers: Vec<CategoryStats>,
    /// 操作系统维度统计，按点击次数降序
    pub operating_systems: Vec<CategoryStats>,
    /// 设备类型维度统计，按点击次数降序
    pub devices: Vec<CategoryStats>,
    /// 点击中 Bot（爬虫/自动化 UA）占比（0-100）
    pub bot_percentage: f64,
    /// 带有效 User-Agent 信息的点击总数，是 `bot_percentage` 的分母
    pub total_with_ua: u64,
}

/// 分类统计
#[derive(Debug, Clone, PartialEq)]
pub struct CategoryStats {
    /// 分类名称（浏览器名、系统名或设备类型）；UA 字段缺失时为 `Unknown`
    pub name: String,
    /// 该分类的点击次数
    pub count: u64,
    /// 占同维度各分类点击数之和的百分比（0-100）
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
            values.push(u64::try_from(row.count).unwrap_or(0));
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

        let limit = u64::from(limit.min(100));
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
                clicks: u64::try_from(row.count).unwrap_or(0),
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

        let limit = u64::from(limit.min(100));
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
        let total: u64 = results
            .iter()
            .map(|r| u64::try_from(r.count).unwrap_or(0))
            .sum();

        let referrer_stats: Vec<ReferrerStats> = results
            .into_iter()
            .map(|row| {
                let referrer = row.referrer.unwrap_or_else(|| "(direct)".to_string());
                let count = u64::try_from(row.count).unwrap_or(0);
                let percentage = percentage(count, total);
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

        let limit = u64::from(limit.min(100));
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
                count: u64::try_from(row.count).unwrap_or(0),
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
            values: trend_rows
                .iter()
                .map(|r| u64::try_from(r.count).unwrap_or(0))
                .collect(),
        };

        // 转换来源统计结果
        let top_referrers: Vec<ReferrerStats> = referrer_rows
            .into_iter()
            .map(|row| {
                let referrer = row.referrer.unwrap_or_else(|| "(direct)".to_string());
                let count = u64::try_from(row.count).unwrap_or(0);
                let percentage = percentage(count, total_clicks);
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
                count: u64::try_from(row.count).unwrap_or(0),
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

        let limit = u64::from(limit.min(20));

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
            let total: u64 = rows
                .iter()
                .map(|r| u64::try_from(r.count).unwrap_or(0))
                .sum();
            rows.into_iter()
                .map(|r| {
                    let count = u64::try_from(r.count).unwrap_or(0);
                    CategoryStats {
                        name: r.field_value.unwrap_or_else(|| "Unknown".to_string()),
                        count,
                        percentage: percentage(count, total),
                    }
                })
                .collect()
        };

        let (bot_count, total_with_ua) = bot_stats;
        let bot_count = u64::try_from(bot_count).unwrap_or(0);
        let total_with_ua = u64::try_from(total_with_ua).unwrap_or(0);
        let bot_percentage = percentage(bot_count, total_with_ua);

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
            total_with_ua,
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

        let limit = u64::from(limit.min(20));

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
            let total: u64 = rows
                .iter()
                .map(|r| u64::try_from(r.count).unwrap_or(0))
                .sum();
            rows.into_iter()
                .map(|r| {
                    let count = u64::try_from(r.count).unwrap_or(0);
                    CategoryStats {
                        name: r.field_value.unwrap_or_else(|| "Unknown".to_string()),
                        count,
                        percentage: percentage(count, total),
                    }
                })
                .collect()
        };

        let (bot_count, total_with_ua) = bot_stats;
        let bot_count = u64::try_from(bot_count).unwrap_or(0);
        let total_with_ua = u64::try_from(total_with_ua).unwrap_or(0);
        let bot_percentage = percentage(bot_count, total_with_ua);

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
            total_with_ua,
        })
    }

    /// 流式导出点击日志（游标分页）
    ///
    /// 返回分批数据流，无总量限制。
    /// 用于大数据量导出场景，避免一次性加载全部数据到内存。
    pub fn export_click_logs_stream(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        batch_size: u64,
    ) -> Pin<
        Box<
            dyn Stream<Item = Result<Vec<migration::entities::click_log::Model>, ShortlinkerError>>
                + Send
                + 'static,
        >,
    > {
        let batch_size = batch_size.max(1);
        Box::pin(
            self.storage
                .stream_click_logs_cursor(start, end, batch_size)
                .map(|result| {
                    result.map_err(|e| {
                        ShortlinkerError::analytics_query_failed(format!(
                            "Failed to export click logs: {}",
                            e
                        ))
                    })
                }),
        )
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
                use std::collections::BTreeMap;

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

                let mut grouped: BTreeMap<String, i64> = BTreeMap::new();
                for row in &daily_results {
                    if let Ok(date) = chrono::NaiveDate::parse_from_str(&row.label, "%Y-%m-%d") {
                        let key = date.format(format_str).to_string();
                        *grouped.entry(key).or_insert(0) += row.count;
                    }
                }

                Ok(grouped
                    .into_iter()
                    .map(|(label, count)| crate::storage::backend::TrendRow { label, count })
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
            values.push(u64::try_from(row.count).unwrap_or(0));
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

        let results = if let GroupBy::Hour = group_by {
            self.storage
                .get_link_trend_from_hourly(code, start, end)
                .await
        } else {
            let start_date = start.date_naive();
            let end_date = end.date_naive();
            self.storage
                .get_link_trend_from_daily(code, start_date, end_date)
                .await
        }
        .map_err(|e| {
            ShortlinkerError::analytics_query_failed(format!("Link trend query failed: {}", e))
        })?;

        let mut labels = Vec::with_capacity(results.len());
        let mut values = Vec::with_capacity(results.len());
        for row in results {
            labels.push(row.label);
            values.push(u64::try_from(row.count).unwrap_or(0));
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

        let total: u64 = results
            .iter()
            .map(|r| u64::try_from(r.count).unwrap_or(0))
            .sum();

        let referrer_stats: Vec<ReferrerStats> = results
            .into_iter()
            .map(|row| {
                let referrer = row.referrer.unwrap_or_else(|| "(direct)".to_string());
                let count = u64::try_from(row.count).unwrap_or(0);
                let percentage = percentage(count, total);
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
                count: u64::try_from(row.count).unwrap_or(0),
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
                clicks: u64::try_from(row.count).unwrap_or(0),
            })
            .collect();

        debug!(
            "Analytics: get_top_links_v2 returned {} links",
            top_links.len()
        );

        Ok(top_links)
    }
}
