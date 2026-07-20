//! 点击统计汇总管理器
//!
//! 负责将原始点击数据聚合到汇总表（hourly/daily），
//! 以及后台任务将小时汇总滚动到天汇总。

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use tracing::{debug, info};

use super::HourlyRollupWriter;
use crate::storage::backend::SeaOrmStorage;
use aster_forge_db::retry::RetryConfig;
use migration::entities::{
    click_stats_daily, click_stats_global_daily, click_stats_global_hourly, click_stats_hourly,
};

/// 点击聚合数据
#[derive(Debug, Clone, Default)]
pub struct ClickAggregation {
    /// 点击计数
    pub count: usize,
    /// 来源统计 (referrer -> count)
    pub referrers: HashMap<String, usize>,
    /// 国家统计 (country -> count)
    pub countries: HashMap<String, usize>,
    /// 流量来源统计 (source -> count)
    pub sources: HashMap<String, usize>,
}

impl ClickAggregation {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            referrers: HashMap::new(),
            countries: HashMap::new(),
            sources: HashMap::new(),
        }
    }

    pub fn merge(&mut self, other: &ClickAggregation) {
        self.count += other.count;
        for (k, v) in &other.referrers {
            *self.referrers.entry(k.clone()).or_insert(0) += v;
        }
        for (k, v) in &other.countries {
            *self.countries.entry(k.clone()).or_insert(0) += v;
        }
        for (k, v) in &other.sources {
            *self.sources.entry(k.clone()).or_insert(0) += v;
        }
    }
}

/// 汇总管理器
pub struct RollupManager {
    storage: Arc<SeaOrmStorage>,
    retry_config: RetryConfig,
}

impl RollupManager {
    pub fn new(storage: Arc<SeaOrmStorage>) -> Self {
        let config = crate::config::get_config();
        let retry_config = RetryConfig {
            max_retries: config.database.retry_count,
            base_delay_ms: config.database.retry_base_delay_ms,
            max_delay_ms: config.database.retry_max_delay_ms,
        };

        Self {
            storage,
            retry_config,
        }
    }

    /// 创建 HourlyRollupWriter 实例
    fn hourly_writer(&self) -> HourlyRollupWriter<'_, sea_orm::DatabaseConnection> {
        HourlyRollupWriter::new(self.storage.get_db(), self.retry_config)
    }

    /// 将时间戳截断到当天开始
    pub fn truncate_to_day(ts: DateTime<Utc>) -> NaiveDate {
        ts.date_naive()
    }

    /// 更新小时汇总（仅计数，无详细信息）
    ///
    /// 在 flush_clicks() 时调用，将点击计数累加到小时汇总表
    pub async fn increment_hourly_counts(
        &self,
        updates: &[(String, usize)],
        timestamp: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        if updates.is_empty() {
            return Ok(());
        }

        let writer = self.hourly_writer();

        writer
            .increment_hourly_counts(updates, timestamp, "rollup")
            .await?;

        let hour_bucket = super::truncate_to_hour(timestamp);
        let total_clicks: usize = updates.iter().map(|(_, count)| count).sum();

        debug!(
            "Hourly rollup updated: {} links, {} clicks (bucket: {})",
            updates.len(),
            total_clicks,
            hour_bucket
        );

        Ok(())
    }

    /// 更新小时汇总（含详细信息）
    ///
    /// 在 log_clicks_batch() 后调用，包含来源和地理信息
    pub async fn increment_hourly_with_details(
        &self,
        aggregated: &HashMap<(String, DateTime<Utc>), ClickAggregation>,
    ) -> anyhow::Result<()> {
        if aggregated.is_empty() {
            return Ok(());
        }

        self.hourly_writer()
            .upsert_hourly_with_details(aggregated, "rollup")
            .await?;

        debug!(
            "Detailed hourly rollup updated: {} records",
            aggregated.len()
        );
        Ok(())
    }

    /// 将小时汇总滚动到天汇总
    ///
    /// 通常由后台任务调用，处理已完成的小时数据
    pub async fn rollup_hourly_to_daily(&self, target_date: NaiveDate) -> anyhow::Result<u64> {
        let db = self.storage.get_db();

        // 获取该日期所有小时汇总
        let start = target_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = (target_date + Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let hourly_records = click_stats_hourly::Entity::find()
            .filter(click_stats_hourly::Column::HourBucket.gte(start))
            .filter(click_stats_hourly::Column::HourBucket.lt(end))
            .all(db)
            .await?;

        if hourly_records.is_empty() {
            return Ok(0);
        }

        // 按 short_code 聚合（预分配容量）
        let mut aggregated: HashMap<String, ClickAggregation> =
            HashMap::with_capacity(hourly_records.len());

        for record in &hourly_records {
            let agg = aggregated
                .entry(record.short_code.clone())
                .or_insert_with(|| ClickAggregation::new(0));

            agg.count = agg
                .count
                .saturating_add(record.click_count.max(0).try_into().unwrap_or(usize::MAX));

            // 合并 referrer 统计
            let referrers = super::parse_json_counts(&record.referrer_counts);
            for (k, v) in referrers {
                *agg.referrers.entry(k).or_insert(0) += v;
            }

            // 合并 country 统计
            let countries = super::parse_json_counts(&record.country_counts);
            for (k, v) in countries {
                *agg.countries.entry(k).or_insert(0) += v;
            }

            // 合并 source 统计
            let sources = super::parse_json_counts(&record.source_counts);
            for (k, v) in sources {
                *agg.sources.entry(k).or_insert(0) += v;
            }
        }

        let mut daily_models = Vec::with_capacity(aggregated.len());
        for (code, agg) in &aggregated {
            let top_referrers = Self::get_top_n(&agg.referrers, 10);
            let top_countries = Self::get_top_n(&agg.countries, 10);
            let top_sources = Self::get_top_n(&agg.sources, 10);
            daily_models.push(click_stats_daily::ActiveModel {
                short_code: Set(code.clone()),
                day_bucket: Set(target_date),
                click_count: Set(i64::try_from(agg.count).unwrap_or(i64::MAX)),
                unique_referrers: Set(Some(i32::try_from(agg.referrers.len()).unwrap_or(i32::MAX))),
                unique_countries: Set(Some(i32::try_from(agg.countries.len()).unwrap_or(i32::MAX))),
                unique_sources: Set(Some(i32::try_from(agg.sources.len()).unwrap_or(i32::MAX))),
                top_referrers: Set(Some(serde_json::to_string(&top_referrers)?)),
                top_countries: Set(Some(serde_json::to_string(&top_countries)?)),
                top_sources: Set(Some(serde_json::to_string(&top_sources)?)),
                ..Default::default()
            });
        }

        let processed = aggregated.len() as u64;

        if !daily_models.is_empty() {
            aster_forge_db::retry::with_sea_orm_retry(
                "rollup_upsert_daily_batch",
                self.retry_config,
                || async {
                    click_stats_daily::Entity::insert_many(daily_models.clone())
                        .on_conflict(
                            OnConflict::columns([
                                click_stats_daily::Column::ShortCode,
                                click_stats_daily::Column::DayBucket,
                            ])
                            .update_columns([
                                click_stats_daily::Column::ClickCount,
                                click_stats_daily::Column::UniqueReferrers,
                                click_stats_daily::Column::UniqueCountries,
                                click_stats_daily::Column::UniqueSources,
                                click_stats_daily::Column::TopReferrers,
                                click_stats_daily::Column::TopCountries,
                                click_stats_daily::Column::TopSources,
                            ])
                            .to_owned(),
                        )
                        .exec(db)
                        .await
                },
            )
            .await?;
            debug!("Upserted {} daily rollup records", daily_models.len());
        }

        // ---- 全局天汇总 ----
        // 从已聚合的 per-code 数据中计算全局统计
        let total_clicks: i64 = aggregated
            .values()
            .map(|a| i64::try_from(a.count).unwrap_or(i64::MAX))
            .fold(0i64, |acc, x| acc.saturating_add(x));
        let unique_links = i32::try_from(aggregated.len()).unwrap_or(i32::MAX);

        let mut global_referrers: HashMap<String, usize> = HashMap::new();
        let mut global_countries: HashMap<String, usize> = HashMap::new();
        let mut global_sources: HashMap<String, usize> = HashMap::new();

        for agg in aggregated.values() {
            for (k, v) in &agg.referrers {
                *global_referrers.entry(k.clone()).or_insert(0) += v;
            }
            for (k, v) in &agg.countries {
                *global_countries.entry(k.clone()).or_insert(0) += v;
            }
            for (k, v) in &agg.sources {
                *global_sources.entry(k.clone()).or_insert(0) += v;
            }
        }

        let top_referrers_json = serde_json::to_string(&Self::get_top_n(&global_referrers, 10))?;
        let top_countries_json = serde_json::to_string(&Self::get_top_n(&global_countries, 10))?;
        let top_sources_json = serde_json::to_string(&Self::get_top_n(&global_sources, 10))?;

        let global_model = click_stats_global_daily::ActiveModel {
            day_bucket: Set(target_date),
            total_clicks: Set(total_clicks),
            unique_links: Set(Some(unique_links)),
            top_referrers: Set(Some(top_referrers_json)),
            top_countries: Set(Some(top_countries_json)),
            top_sources: Set(Some(top_sources_json)),
            ..Default::default()
        };
        aster_forge_db::retry::with_sea_orm_retry(
            "rollup_upsert_global_daily",
            self.retry_config,
            || async {
                click_stats_global_daily::Entity::insert(global_model.clone())
                    .on_conflict(
                        OnConflict::column(click_stats_global_daily::Column::DayBucket)
                            .update_columns([
                                click_stats_global_daily::Column::TotalClicks,
                                click_stats_global_daily::Column::UniqueLinks,
                                click_stats_global_daily::Column::TopReferrers,
                                click_stats_global_daily::Column::TopCountries,
                                click_stats_global_daily::Column::TopSources,
                            ])
                            .to_owned(),
                    )
                    .exec(db)
                    .await
            },
        )
        .await?;

        info!(
            "Hourly-to-daily rollup completed: {} links (date: {})",
            processed, target_date
        );
        Ok(processed)
    }

    /// 清理过期的汇总数据
    pub async fn cleanup_expired(
        &self,
        hourly_retention_days: u64,
        daily_retention_days: u64,
    ) -> anyhow::Result<(u64, u64)> {
        let db = self.storage.get_db();
        let now = Utc::now();

        // 清理过期的小时汇总
        let hourly_cutoff = now - Duration::days(hourly_retention_days as i64);
        let hourly_deleted = click_stats_hourly::Entity::delete_many()
            .filter(click_stats_hourly::Column::HourBucket.lt(hourly_cutoff))
            .exec(db)
            .await?
            .rows_affected;

        // 清理过期的天汇总
        let daily_cutoff = Self::truncate_to_day(now - Duration::days(daily_retention_days as i64));
        let daily_deleted = click_stats_daily::Entity::delete_many()
            .filter(click_stats_daily::Column::DayBucket.lt(daily_cutoff))
            .exec(db)
            .await?
            .rows_affected;

        // 清理过期的全局小时汇总
        click_stats_global_hourly::Entity::delete_many()
            .filter(click_stats_global_hourly::Column::HourBucket.lt(hourly_cutoff))
            .exec(db)
            .await?;

        // 清理过期的全局天汇总
        click_stats_global_daily::Entity::delete_many()
            .filter(click_stats_global_daily::Column::DayBucket.lt(daily_cutoff))
            .exec(db)
            .await?;

        info!(
            "Rollup cleanup completed: hourly {} rows, daily {} rows",
            hourly_deleted, daily_deleted
        );

        Ok((hourly_deleted, daily_deleted))
    }

    // ============ 辅助方法 ============

    fn get_top_n(map: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
        let mut items: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        items.sort_by_key(|item| std::cmp::Reverse(item.1));
        items.truncate(n);
        items
    }
}

/// 从 ClickDetail 列表聚合数据
pub fn aggregate_click_details(
    details: &[crate::analytics::ClickDetail],
) -> HashMap<(String, DateTime<Utc>), ClickAggregation> {
    let mut result: HashMap<(String, DateTime<Utc>), ClickAggregation> = HashMap::new();

    for detail in details {
        let hour_bucket = super::truncate_to_hour(detail.timestamp);
        let key = (detail.code.clone(), hour_bucket);

        let agg = result
            .entry(key)
            .or_insert_with(|| ClickAggregation::new(0));
        agg.count += 1;

        if let Some(ref referrer) = detail.referrer {
            let referrer_key = if referrer.is_empty() {
                "direct".to_string()
            } else {
                referrer.clone()
            };
            *agg.referrers.entry(referrer_key).or_insert(0) += 1;
        } else {
            *agg.referrers.entry("direct".to_string()).or_insert(0) += 1;
        }

        if let Some(ref country) = detail.country {
            *agg.countries.entry(country.clone()).or_insert(0) += 1;
        } else {
            *agg.countries.entry("Unknown".to_string()).or_insert(0) += 1;
        }

        // 聚合 source
        if let Some(ref source) = detail.source {
            *agg.sources.entry(source.clone()).or_insert(0) += 1;
        } else {
            *agg.sources.entry("direct".to_string()).or_insert(0) += 1;
        }
    }

    result
}
