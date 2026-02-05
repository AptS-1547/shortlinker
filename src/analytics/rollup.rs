//! 点击统计汇总管理器
//!
//! 负责将原始点击数据聚合到汇总表（hourly/daily），
//! 以及后台任务将小时汇总滚动到天汇总。

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use tracing::{debug, info};

use crate::storage::backend::SeaOrmStorage;
use crate::storage::backend::retry::{self, RetryConfig};
use migration::entities::{click_stats_daily, click_stats_global_hourly, click_stats_hourly};

/// 点击聚合数据
#[derive(Debug, Clone, Default)]
pub struct ClickAggregation {
    /// 点击计数
    pub count: usize,
    /// 来源统计 (referrer -> count)
    pub referrers: HashMap<String, usize>,
    /// 国家统计 (country -> count)
    pub countries: HashMap<String, usize>,
}

impl ClickAggregation {
    pub fn new(count: usize) -> Self {
        Self {
            count,
            referrers: HashMap::new(),
            countries: HashMap::new(),
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

        let hour_bucket = super::truncate_to_hour(timestamp);
        let db = self.storage.get_db();

        for (code, count) in updates {
            // 尝试查找现有记录
            let existing = click_stats_hourly::Entity::find()
                .filter(click_stats_hourly::Column::ShortCode.eq(code.as_str()))
                .filter(click_stats_hourly::Column::HourBucket.eq(hour_bucket))
                .one(db)
                .await?;

            if let Some(record) = existing {
                // 更新现有记录
                let mut active: click_stats_hourly::ActiveModel = record.into();
                if let Set(old_count) = active.click_count {
                    active.click_count = Set(old_count + *count as i64);
                }
                retry::with_retry("update_hourly_stats", self.retry_config, || async {
                    click_stats_hourly::Entity::update(active.clone())
                        .exec(db)
                        .await
                })
                .await?;
            } else {
                // 插入新记录
                let model = click_stats_hourly::ActiveModel {
                    short_code: Set(code.clone()),
                    hour_bucket: Set(hour_bucket),
                    click_count: Set(*count as i64),
                    referrer_counts: Set(None),
                    country_counts: Set(None),
                    ..Default::default()
                };
                retry::with_retry("insert_hourly_stats", self.retry_config, || async {
                    click_stats_hourly::Entity::insert(model.clone())
                        .exec(db)
                        .await
                })
                .await?;
            }
        }

        // 同时更新全局小时汇总
        let total_clicks: usize = updates.iter().map(|(_, c)| c).sum();
        let unique_links = updates.len() as i32;
        self.update_global_hourly(hour_bucket, total_clicks, unique_links)
            .await?;

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

        let db = self.storage.get_db();

        for ((code, hour_bucket), agg) in aggregated {
            // 尝试查找现有记录
            let existing = click_stats_hourly::Entity::find()
                .filter(click_stats_hourly::Column::ShortCode.eq(code.as_str()))
                .filter(click_stats_hourly::Column::HourBucket.eq(*hour_bucket))
                .one(db)
                .await?;

            if let Some(record) = existing {
                // 合并现有数据
                let mut merged_referrers = super::parse_json_counts(&record.referrer_counts);
                let mut merged_countries = super::parse_json_counts(&record.country_counts);

                for (k, v) in &agg.referrers {
                    *merged_referrers.entry(k.clone()).or_insert(0) += v;
                }
                for (k, v) in &agg.countries {
                    *merged_countries.entry(k.clone()).or_insert(0) += v;
                }

                let mut active: click_stats_hourly::ActiveModel = record.into();
                if let Set(old_count) = active.click_count {
                    active.click_count = Set(old_count + agg.count as i64);
                }
                active.referrer_counts = Set(Some(super::to_json_string(&merged_referrers)));
                active.country_counts = Set(Some(super::to_json_string(&merged_countries)));

                retry::with_retry("update_hourly_detailed", self.retry_config, || async {
                    click_stats_hourly::Entity::update(active.clone())
                        .exec(db)
                        .await
                })
                .await?;
            } else {
                // 插入新记录
                let model = click_stats_hourly::ActiveModel {
                    short_code: Set(code.clone()),
                    hour_bucket: Set(*hour_bucket),
                    click_count: Set(agg.count as i64),
                    referrer_counts: Set(Some(super::to_json_string(&agg.referrers))),
                    country_counts: Set(Some(super::to_json_string(&agg.countries))),
                    ..Default::default()
                };
                retry::with_retry("insert_hourly_detailed", self.retry_config, || async {
                    click_stats_hourly::Entity::insert(model.clone())
                        .exec(db)
                        .await
                })
                .await?;
            }
        }

        debug!(
            "Detailed hourly rollup updated: {} records",
            aggregated.len()
        );
        Ok(())
    }

    /// 更新全局小时汇总
    async fn update_global_hourly(
        &self,
        hour_bucket: DateTime<Utc>,
        clicks: usize,
        unique_links: i32,
    ) -> anyhow::Result<()> {
        let db = self.storage.get_db();

        let existing = click_stats_global_hourly::Entity::find()
            .filter(click_stats_global_hourly::Column::HourBucket.eq(hour_bucket))
            .one(db)
            .await?;

        if let Some(record) = existing {
            let mut active: click_stats_global_hourly::ActiveModel = record.into();
            if let Set(old_clicks) = active.total_clicks {
                active.total_clicks = Set(old_clicks + clicks as i64);
            }
            // unique_links 不做累加（会导致重复计数），留给 rollup_hourly_to_daily 从 hourly 表统计
            retry::with_retry("update_global_hourly", self.retry_config, || async {
                click_stats_global_hourly::Entity::update(active.clone())
                    .exec(db)
                    .await
            })
            .await?;
        } else {
            let model = click_stats_global_hourly::ActiveModel {
                hour_bucket: Set(hour_bucket),
                total_clicks: Set(clicks as i64),
                unique_links: Set(Some(unique_links)),
                top_referrers: Set(None),
                top_countries: Set(None),
                ..Default::default()
            };
            retry::with_retry("insert_global_hourly", self.retry_config, || async {
                click_stats_global_hourly::Entity::insert(model.clone())
                    .exec(db)
                    .await
            })
            .await?;
        }

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

        // 按 short_code 聚合
        let mut aggregated: HashMap<String, ClickAggregation> = HashMap::new();

        for record in &hourly_records {
            let agg = aggregated
                .entry(record.short_code.clone())
                .or_insert_with(|| ClickAggregation::new(0));

            agg.count += record.click_count as usize;

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
        }

        // 写入天汇总
        let mut processed = 0u64;
        for (code, agg) in aggregated {
            let top_referrers = Self::get_top_n(&agg.referrers, 10);
            let top_countries = Self::get_top_n(&agg.countries, 10);

            // 查找或创建天汇总记录
            let existing = click_stats_daily::Entity::find()
                .filter(click_stats_daily::Column::ShortCode.eq(&code))
                .filter(click_stats_daily::Column::DayBucket.eq(target_date))
                .one(db)
                .await?;

            if let Some(record) = existing {
                let mut active: click_stats_daily::ActiveModel = record.into();
                if let Set(old_count) = active.click_count {
                    active.click_count = Set(old_count + agg.count as i64);
                }
                active.unique_referrers = Set(Some(agg.referrers.len() as i32));
                active.unique_countries = Set(Some(agg.countries.len() as i32));
                active.top_referrers = Set(Some(serde_json::to_string(&top_referrers)?));
                active.top_countries = Set(Some(serde_json::to_string(&top_countries)?));

                retry::with_retry("rollup_update_daily", self.retry_config, || async {
                    click_stats_daily::Entity::update(active.clone())
                        .exec(db)
                        .await
                })
                .await?;
            } else {
                let model = click_stats_daily::ActiveModel {
                    short_code: Set(code),
                    day_bucket: Set(target_date),
                    click_count: Set(agg.count as i64),
                    unique_referrers: Set(Some(agg.referrers.len() as i32)),
                    unique_countries: Set(Some(agg.countries.len() as i32)),
                    top_referrers: Set(Some(serde_json::to_string(&top_referrers)?)),
                    top_countries: Set(Some(serde_json::to_string(&top_countries)?)),
                    ..Default::default()
                };
                retry::with_retry("rollup_insert_daily", self.retry_config, || async {
                    click_stats_daily::Entity::insert(model.clone())
                        .exec(db)
                        .await
                })
                .await?;
            }

            processed += 1;
        }

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

        info!(
            "Rollup cleanup completed: hourly {} rows, daily {} rows",
            hourly_deleted, daily_deleted
        );

        Ok((hourly_deleted, daily_deleted))
    }

    // ============ 辅助方法 ============

    fn get_top_n(map: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
        let mut items: Vec<_> = map.iter().map(|(k, v)| (k.clone(), *v)).collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));
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
    }

    result
}
