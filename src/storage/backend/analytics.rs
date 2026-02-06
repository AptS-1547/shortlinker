//! Analytics 相关的数据库查询
//!
//! 提供点击日志的统计查询方法，供 AnalyticsService 调用。
//! 包含两套查询方法：
//! - 原始查询（从 click_logs 表聚合）
//! - v2 查询（从汇总表读取，性能更好）

use std::collections::HashMap;
use std::pin::Pin;

use chrono::{DateTime, NaiveDate, Utc};
use futures_util::stream::Stream;
use sea_orm::{
    ColumnTrait, EntityTrait, ExprTrait, FromQueryResult, JoinType, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect, sea_query::Expr,
};
use tracing::warn;

use migration::entities::{
    click_log, click_stats_daily, click_stats_global_hourly, click_stats_hourly, user_agent,
};

/// 游标分页的点击日志流类型
type ClickLogStream = Pin<
    Box<dyn Stream<Item = anyhow::Result<(Vec<click_log::Model>, Option<i64>)>> + Send + 'static>,
>;

// ============ 查询结果类型 ============

/// 趋势查询结果行
#[derive(Debug, FromQueryResult, Clone)]
pub struct TrendRow {
    pub label: String,
    pub count: i64,
}

/// 来源查询结果行
#[derive(Debug, FromQueryResult, Clone)]
pub struct ReferrerRow {
    pub referrer: Option<String>,
    pub count: i64,
}

/// 地理位置查询结果行
#[derive(Debug, FromQueryResult, Clone)]
pub struct GeoRow {
    pub country: Option<String>,
    pub city: Option<String>,
    pub count: i64,
}

/// 热门链接查询结果行
#[derive(Debug, FromQueryResult, Clone)]
pub struct TopLinkRow {
    pub short_code: String,
    pub count: i64,
}

/// UA 统计查询结果行
#[derive(Debug, FromQueryResult, Clone)]
pub struct UaStatsRow {
    pub field_value: Option<String>,
    pub count: i64,
}

/// Bot 统计原始查询结果
#[derive(Debug, FromQueryResult)]
struct BotStatsRaw {
    pub bot_count: i64,
    pub total: i64,
}

/// 分组方式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupBy {
    Hour,
    Day,
    Week,
    Month,
}

// ============ SeaOrmStorage Analytics 方法 ============

impl super::SeaOrmStorage {
    // ============ 原始查询（从 click_logs 表） ============

    /// 统计指定链接的点击数
    pub async fn count_link_clicks(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<u64> {
        click_log::Entity::find()
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .count(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取链接点击趋势
    pub async fn get_link_trend(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        date_expr: Expr,
    ) -> anyhow::Result<Vec<TrendRow>> {
        click_log::Entity::find()
            .select_only()
            .column_as(date_expr.clone(), "label")
            .column_as(click_log::Column::Id.count(), "count")
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(date_expr.clone())
            .order_by_asc(date_expr)
            .into_model::<TrendRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取链接来源统计
    ///
    /// 使用 `source` 字段（utm_source 参数、ref:{domain} 或 direct）
    pub async fn get_link_referrers(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<ReferrerRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column_as(click_log::Column::Source, "referrer")
            .column_as(count_expr.clone(), "count")
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(click_log::Column::Source)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<ReferrerRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取链接地理分布
    pub async fn get_link_geo(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<GeoRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column(click_log::Column::Country)
            .column(click_log::Column::City)
            .column_as(count_expr.clone(), "count")
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .filter(click_log::Column::Country.is_not_null())
            .group_by(click_log::Column::Country)
            .group_by(click_log::Column::City)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<GeoRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    // ============ 全局统计查询（不限定 code） ============

    /// 获取全局点击趋势
    pub async fn get_global_trend(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        date_expr: Expr,
    ) -> anyhow::Result<Vec<TrendRow>> {
        click_log::Entity::find()
            .select_only()
            .column_as(date_expr.clone(), "label")
            .column_as(click_log::Column::Id.count(), "count")
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(date_expr.clone())
            .order_by_asc(date_expr)
            .into_model::<TrendRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取热门链接
    pub async fn get_top_links(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<TopLinkRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column(click_log::Column::ShortCode)
            .column_as(count_expr.clone(), "count")
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(click_log::Column::ShortCode)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<TopLinkRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取全局来源统计
    ///
    /// 使用 `source` 字段（utm_source 参数、ref:{domain} 或 direct）
    pub async fn get_global_referrers(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<ReferrerRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column_as(click_log::Column::Source, "referrer")
            .column_as(count_expr.clone(), "count")
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(click_log::Column::Source)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<ReferrerRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取全局地理分布
    pub async fn get_global_geo(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<GeoRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column(click_log::Column::Country)
            .column(click_log::Column::City)
            .column_as(count_expr.clone(), "count")
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .filter(click_log::Column::Country.is_not_null())
            .group_by(click_log::Column::Country)
            .group_by(click_log::Column::City)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<GeoRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    // ============ UA/设备统计查询（JOIN user_agents 表） ============

    /// 构建 click_logs -> user_agents 的 JOIN 关系
    fn click_log_to_user_agent_relation() -> sea_orm::RelationDef {
        click_log::Entity::belongs_to(user_agent::Entity)
            .from(click_log::Column::UserAgentHash)
            .to(user_agent::Column::Hash)
            .into()
    }

    /// 获取浏览器统计
    pub async fn get_browser_stats(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<UaStatsRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column_as(user_agent::Column::BrowserName, "field_value")
            .column_as(count_expr.clone(), "count")
            .join(
                JoinType::InnerJoin,
                Self::click_log_to_user_agent_relation(),
            )
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .filter(user_agent::Column::BrowserName.is_not_null())
            .group_by(user_agent::Column::BrowserName)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<UaStatsRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取操作系统统计
    pub async fn get_os_stats(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<UaStatsRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column_as(user_agent::Column::OsName, "field_value")
            .column_as(count_expr.clone(), "count")
            .join(
                JoinType::InnerJoin,
                Self::click_log_to_user_agent_relation(),
            )
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .filter(user_agent::Column::OsName.is_not_null())
            .group_by(user_agent::Column::OsName)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<UaStatsRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取设备类型统计
    pub async fn get_device_stats(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<UaStatsRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column_as(user_agent::Column::DeviceCategory, "field_value")
            .column_as(count_expr.clone(), "count")
            .join(
                JoinType::InnerJoin,
                Self::click_log_to_user_agent_relation(),
            )
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .filter(user_agent::Column::DeviceCategory.is_not_null())
            .group_by(user_agent::Column::DeviceCategory)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<UaStatsRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取指定链接的浏览器统计
    pub async fn get_link_browser_stats(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<UaStatsRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column_as(user_agent::Column::BrowserName, "field_value")
            .column_as(count_expr.clone(), "count")
            .join(
                JoinType::InnerJoin,
                Self::click_log_to_user_agent_relation(),
            )
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .filter(user_agent::Column::BrowserName.is_not_null())
            .group_by(user_agent::Column::BrowserName)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<UaStatsRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取指定链接的操作系统统计
    pub async fn get_link_os_stats(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<UaStatsRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column_as(user_agent::Column::OsName, "field_value")
            .column_as(count_expr.clone(), "count")
            .join(
                JoinType::InnerJoin,
                Self::click_log_to_user_agent_relation(),
            )
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .filter(user_agent::Column::OsName.is_not_null())
            .group_by(user_agent::Column::OsName)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<UaStatsRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取指定链接的设备类型统计
    pub async fn get_link_device_stats(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<UaStatsRow>> {
        let count_expr = click_log::Column::Id.count();
        click_log::Entity::find()
            .select_only()
            .column_as(user_agent::Column::DeviceCategory, "field_value")
            .column_as(count_expr.clone(), "count")
            .join(
                JoinType::InnerJoin,
                Self::click_log_to_user_agent_relation(),
            )
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .filter(user_agent::Column::DeviceCategory.is_not_null())
            .group_by(user_agent::Column::DeviceCategory)
            .order_by_desc(count_expr)
            .limit(limit)
            .into_model::<UaStatsRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取指定链接的 Bot 统计 (bot_count, total_with_ua)
    pub async fn get_link_bot_stats(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<(i64, i64)> {
        // SUM(CASE WHEN is_bot = true THEN 1 ELSE 0 END)
        let bot_sum = Expr::case(user_agent::Column::IsBot.eq(true), 1)
            .finally(0)
            .sum();

        let result = click_log::Entity::find()
            .select_only()
            .column_as(bot_sum, "bot_count")
            .column_as(click_log::Column::Id.count(), "total")
            .join(
                JoinType::InnerJoin,
                Self::click_log_to_user_agent_relation(),
            )
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .into_model::<BotStatsRaw>()
            .one(&self.db)
            .await?;

        match result {
            Some(row) => Ok((row.bot_count, row.total)),
            None => Ok((0, 0)),
        }
    }

    /// 获取 Bot 统计 (bot_count, total_with_ua)
    pub async fn get_bot_stats(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<(i64, i64)> {
        // SUM(CASE WHEN is_bot = true THEN 1 ELSE 0 END)
        let bot_sum = Expr::case(user_agent::Column::IsBot.eq(true), 1)
            .finally(0)
            .sum();

        let result = click_log::Entity::find()
            .select_only()
            .column_as(bot_sum, "bot_count")
            .column_as(click_log::Column::Id.count(), "total")
            .join(
                JoinType::InnerJoin,
                Self::click_log_to_user_agent_relation(),
            )
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .into_model::<BotStatsRaw>()
            .one(&self.db)
            .await?;

        match result {
            Some(row) => Ok((row.bot_count, row.total)),
            None => Ok((0, 0)),
        }
    }

    // ============ v2 查询（从汇总表读取） ============

    /// 从小时汇总表获取链接点击趋势
    pub async fn get_link_trend_from_hourly(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<TrendRow>> {
        let records = click_stats_hourly::Entity::find()
            .filter(click_stats_hourly::Column::ShortCode.eq(code))
            .filter(click_stats_hourly::Column::HourBucket.gte(start))
            .filter(click_stats_hourly::Column::HourBucket.lte(end))
            .order_by_asc(click_stats_hourly::Column::HourBucket)
            .all(&self.db)
            .await?;

        Ok(records
            .into_iter()
            .map(|r| TrendRow {
                label: r.hour_bucket.format("%Y-%m-%d %H:00").to_string(),
                count: r.click_count,
            })
            .collect())
    }

    /// 从天汇总表获取链接点击趋势
    pub async fn get_link_trend_from_daily(
        &self,
        code: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> anyhow::Result<Vec<TrendRow>> {
        let records = click_stats_daily::Entity::find()
            .filter(click_stats_daily::Column::ShortCode.eq(code))
            .filter(click_stats_daily::Column::DayBucket.gte(start))
            .filter(click_stats_daily::Column::DayBucket.lte(end))
            .order_by_asc(click_stats_daily::Column::DayBucket)
            .all(&self.db)
            .await?;

        Ok(records
            .into_iter()
            .map(|r| TrendRow {
                label: r.day_bucket.format("%Y-%m-%d").to_string(),
                count: r.click_count,
            })
            .collect())
    }

    /// 从全局小时汇总表获取趋势
    pub async fn get_global_trend_from_hourly(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<TrendRow>> {
        let records = click_stats_global_hourly::Entity::find()
            .filter(click_stats_global_hourly::Column::HourBucket.gte(start))
            .filter(click_stats_global_hourly::Column::HourBucket.lte(end))
            .order_by_asc(click_stats_global_hourly::Column::HourBucket)
            .all(&self.db)
            .await?;

        Ok(records
            .into_iter()
            .map(|r| TrendRow {
                label: r.hour_bucket.format("%Y-%m-%d %H:00").to_string(),
                count: r.total_clicks,
            })
            .collect())
    }

    /// 从小时汇总表获取来源统计
    ///
    /// 使用 `source_counts` 字段（utm_source 参数、ref:{domain} 或 direct）
    ///
    /// 注意：此方法在内存中聚合 JSON 字段，大时间范围查询可能影响性能。
    /// 建议时间范围不超过 30 天。
    pub async fn get_link_referrers_from_rollup(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: usize,
    ) -> anyhow::Result<Vec<ReferrerRow>> {
        // 时间范围检查
        let duration = end - start;
        if duration.num_days() > 30 {
            warn!(
                "get_link_referrers_from_rollup: Large time range ({} days) may cause memory issues",
                duration.num_days()
            );
        }

        // 限制查询记录数，防止内存溢出（90天 * 24小时 = 2160）
        const MAX_HOURLY_RECORDS: u64 = 2160;

        let records = click_stats_hourly::Entity::find()
            .filter(click_stats_hourly::Column::ShortCode.eq(code))
            .filter(click_stats_hourly::Column::HourBucket.gte(start))
            .filter(click_stats_hourly::Column::HourBucket.lte(end))
            .limit(MAX_HOURLY_RECORDS)
            .all(&self.db)
            .await?;

        if records.len() >= MAX_HOURLY_RECORDS as usize {
            warn!(
                "get_link_referrers_from_rollup: Query hit limit ({} records), results may be incomplete",
                MAX_HOURLY_RECORDS
            );
        }

        // 聚合所有记录的 source 统计
        let mut aggregated: HashMap<String, i64> = HashMap::new();
        for record in records {
            if let Some(ref json_str) = record.source_counts
                && let Ok(counts) = serde_json::from_str::<HashMap<String, i64>>(json_str)
            {
                for (k, v) in counts {
                    *aggregated.entry(k).or_insert(0) += v;
                }
            }
        }

        // 排序并取 top N
        let mut items: Vec<_> = aggregated.into_iter().collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(limit);

        Ok(items
            .into_iter()
            .map(|(source, count)| ReferrerRow {
                referrer: Some(source),
                count,
            })
            .collect())
    }

    /// 从小时汇总表获取地理分布
    ///
    /// 注意：此方法在内存中聚合 JSON 字段，大时间范围查询可能影响性能。
    /// 建议时间范围不超过 30 天。
    pub async fn get_link_geo_from_rollup(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: usize,
    ) -> anyhow::Result<Vec<GeoRow>> {
        // 时间范围检查
        let duration = end - start;
        if duration.num_days() > 30 {
            warn!(
                "get_link_geo_from_rollup: Large time range ({} days) may cause memory issues",
                duration.num_days()
            );
        }

        // 限制查询记录数，防止内存溢出（90天 * 24小时 = 2160）
        const MAX_HOURLY_RECORDS: u64 = 2160;

        let records = click_stats_hourly::Entity::find()
            .filter(click_stats_hourly::Column::ShortCode.eq(code))
            .filter(click_stats_hourly::Column::HourBucket.gte(start))
            .filter(click_stats_hourly::Column::HourBucket.lte(end))
            .limit(MAX_HOURLY_RECORDS)
            .all(&self.db)
            .await?;

        if records.len() >= MAX_HOURLY_RECORDS as usize {
            warn!(
                "get_link_geo_from_rollup: Query hit limit ({} records), results may be incomplete",
                MAX_HOURLY_RECORDS
            );
        }

        // 聚合所有记录的 country 统计
        let mut aggregated: HashMap<String, i64> = HashMap::new();
        for record in records {
            if let Some(ref json_str) = record.country_counts
                && let Ok(counts) = serde_json::from_str::<HashMap<String, i64>>(json_str)
            {
                for (k, v) in counts {
                    *aggregated.entry(k).or_insert(0) += v;
                }
            }
        }

        // 排序并取 top N
        let mut items: Vec<_> = aggregated.into_iter().collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
        items.truncate(limit);

        Ok(items
            .into_iter()
            .map(|(country, count)| GeoRow {
                country: if country == "Unknown" {
                    None
                } else {
                    Some(country)
                },
                city: None, // 汇总表不存储城市级别信息
                count,
            })
            .collect())
    }

    /// 从天汇总表获取热门链接
    ///
    /// 使用 SQL GROUP BY + SUM 聚合，性能优于内存聚合。
    pub async fn get_top_links_from_daily(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        limit: usize,
    ) -> anyhow::Result<Vec<TopLinkRow>> {
        let count_expr = click_stats_daily::Column::ClickCount.sum();

        let results = click_stats_daily::Entity::find()
            .select_only()
            .column(click_stats_daily::Column::ShortCode)
            .column_as(count_expr.clone(), "count")
            .filter(click_stats_daily::Column::DayBucket.gte(start))
            .filter(click_stats_daily::Column::DayBucket.lte(end))
            .group_by(click_stats_daily::Column::ShortCode)
            .order_by_desc(count_expr)
            .limit(limit as u64)
            .into_model::<TopLinkRow>()
            .all(&self.db)
            .await?;

        Ok(results)
    }

    // ============ 导出与分页 ============

    /// 导出点击日志
    pub async fn export_click_logs(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<click_log::Model>> {
        click_log::Entity::find()
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .order_by_desc(click_log::Column::ClickedAt)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 流式导出点击日志（OFFSET 分页，兼容旧版）
    pub fn stream_click_logs_paginated(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        page_size: u64,
    ) -> Pin<Box<dyn Stream<Item = anyhow::Result<Vec<click_log::Model>>> + Send + 'static>> {
        let db = self.db.clone();

        use futures_util::stream;

        Box::pin(stream::unfold(
            (0u64, db, start, end, page_size),
            |(page, db, start, end, page_size)| async move {
                let models = click_log::Entity::find()
                    .filter(click_log::Column::ClickedAt.gte(start))
                    .filter(click_log::Column::ClickedAt.lte(end))
                    .order_by_desc(click_log::Column::ClickedAt)
                    .limit(page_size)
                    .offset(page * page_size)
                    .all(&db)
                    .await;

                match models {
                    Ok(models) if models.is_empty() => None,
                    Ok(models) => Some((Ok(models), (page + 1, db, start, end, page_size))),
                    Err(e) => Some((
                        Err(anyhow::anyhow!(
                            "Paginated query failed (page={}): {}",
                            page,
                            e
                        )),
                        (page + 1, db, start, end, page_size),
                    )),
                }
            },
        ))
    }

    /// 流式导出点击日志（游标分页，性能更好）
    ///
    /// 使用 ID 作为游标，避免 OFFSET 在大数据量下的性能问题。
    /// 返回 (数据, 下一个游标) 的流。
    pub fn stream_click_logs_cursor(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        page_size: u64,
    ) -> ClickLogStream {
        let db = self.db.clone();

        use futures_util::stream;

        Box::pin(stream::unfold(
            (None::<i64>, db, start, end, page_size, false),
            |(cursor, db, start, end, page_size, done)| async move {
                if done {
                    return None;
                }

                let mut query = click_log::Entity::find()
                    .filter(click_log::Column::ClickedAt.gte(start))
                    .filter(click_log::Column::ClickedAt.lte(end));

                // 如果有游标，从游标位置开始
                if let Some(last_id) = cursor {
                    query = query.filter(click_log::Column::Id.gt(last_id));
                }

                let models = query
                    .order_by_asc(click_log::Column::Id)
                    .limit(page_size)
                    .all(&db)
                    .await;

                match models {
                    Ok(models) if models.is_empty() => None,
                    Ok(models) => {
                        let next_cursor = models.last().map(|m| m.id);
                        let is_last = (models.len() as u64) < page_size;
                        Some((
                            Ok((models, next_cursor)),
                            (next_cursor, db, start, end, page_size, is_last),
                        ))
                    }
                    Err(e) => Some((
                        Err(anyhow::anyhow!("Cursor pagination query failed: {}", e)),
                        (cursor, db, start, end, page_size, true),
                    )),
                }
            },
        ))
    }
}
