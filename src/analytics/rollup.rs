//! 点击统计汇总管理器
//!
//! 负责将原始点击数据聚合到汇总表（hourly/daily），
//! 以及后台任务将小时汇总滚动到天汇总。

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration, NaiveDate, Utc};
use sea_orm::sea_query::{CaseStatement, Expr, SimpleExpr};
use sea_orm::{ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use tracing::{debug, info};

use super::HourlyRollupWriter;
use crate::storage::backend::SeaOrmStorage;
use crate::storage::backend::retry::{self, RetryConfig};
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
    fn hourly_writer(&self) -> HourlyRollupWriter<'_, impl sea_orm::ConnectionTrait> {
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

        let hour_bucket = super::truncate_to_hour(timestamp);
        let writer = self.hourly_writer();

        // 更新各链接的小时汇总
        writer
            .upsert_hourly_counts(updates, timestamp, "rollup")
            .await?;

        // 同时更新全局小时汇总
        let total_clicks: usize = updates.iter().map(|(_, c)| c).sum();
        let unique_links = updates.len() as i32;
        writer
            .upsert_global_hourly(hour_bucket, total_clicks, unique_links, "rollup")
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

            // 合并 source 统计
            let sources = super::parse_json_counts(&record.source_counts);
            for (k, v) in sources {
                *agg.sources.entry(k).or_insert(0) += v;
            }
        }

        // 写入天汇总
        // 先批量查询现有记录，避免 N+1 查询
        let codes: Vec<&String> = aggregated.keys().collect();
        let existing_records = click_stats_daily::Entity::find()
            .filter(click_stats_daily::Column::ShortCode.is_in(codes.iter().map(|s| s.as_str())))
            .filter(click_stats_daily::Column::DayBucket.eq(target_date))
            .all(db)
            .await?;

        let existing_map: HashMap<String, click_stats_daily::Model> = existing_records
            .into_iter()
            .map(|r| (r.short_code.clone(), r))
            .collect();

        // 分离需要插入和更新的记录
        let mut to_insert: Vec<click_stats_daily::ActiveModel> = Vec::new();
        let mut to_update: Vec<click_stats_daily::ActiveModel> = Vec::new();

        for (code, agg) in &aggregated {
            let top_referrers = Self::get_top_n(&agg.referrers, 10);
            let top_countries = Self::get_top_n(&agg.countries, 10);
            let top_sources = Self::get_top_n(&agg.sources, 10);

            // 从 HashMap 查找现有记录
            if let Some(record) = existing_map.get(code) {
                let mut active: click_stats_daily::ActiveModel = record.clone().into();
                if let Set(old_count) = active.click_count {
                    active.click_count = Set(old_count + agg.count as i64);
                }
                active.unique_referrers = Set(Some(agg.referrers.len() as i32));
                active.unique_countries = Set(Some(agg.countries.len() as i32));
                active.unique_sources = Set(Some(agg.sources.len() as i32));
                active.top_referrers = Set(Some(serde_json::to_string(&top_referrers)?));
                active.top_countries = Set(Some(serde_json::to_string(&top_countries)?));
                active.top_sources = Set(Some(serde_json::to_string(&top_sources)?));
                to_update.push(active);
            } else {
                let model = click_stats_daily::ActiveModel {
                    short_code: Set(code.clone()),
                    day_bucket: Set(target_date),
                    click_count: Set(agg.count as i64),
                    unique_referrers: Set(Some(agg.referrers.len() as i32)),
                    unique_countries: Set(Some(agg.countries.len() as i32)),
                    unique_sources: Set(Some(agg.sources.len() as i32)),
                    top_referrers: Set(Some(serde_json::to_string(&top_referrers)?)),
                    top_countries: Set(Some(serde_json::to_string(&top_countries)?)),
                    top_sources: Set(Some(serde_json::to_string(&top_sources)?)),
                    ..Default::default()
                };
                to_insert.push(model);
            }
        }

        let processed = aggregated.len() as u64;

        // 批量插入新记录
        if !to_insert.is_empty() {
            let insert_count = to_insert.len();
            retry::with_retry("rollup_insert_daily_batch", self.retry_config, || async {
                click_stats_daily::Entity::insert_many(to_insert.clone())
                    .exec(db)
                    .await
            })
            .await?;
            debug!("Batch inserted {} daily rollup records", insert_count);
        }

        // 批量更新现有记录（使用 CASE WHEN 替代逐条 UPDATE）
        if !to_update.is_empty() {
            const BATCH_SIZE: usize = 100;
            for chunk in to_update.chunks(BATCH_SIZE) {
                retry::with_retry("rollup_update_daily_batch", self.retry_config, || async {
                    self.batch_update_daily(db, chunk).await
                })
                .await?;
            }
            debug!("Batch updated {} daily rollup records", to_update.len());
        }

        // ---- 全局天汇总 ----
        // 从已聚合的 per-code 数据中计算全局统计
        let total_clicks: i64 = aggregated.values().map(|a| a.count as i64).sum();
        let unique_links = aggregated.len() as i32;

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

        // 覆盖式 upsert（rollup 是从 hourly 完整重算的，应该幂等）
        let existing_global = click_stats_global_daily::Entity::find()
            .filter(click_stats_global_daily::Column::DayBucket.eq(target_date))
            .one(db)
            .await?;

        if let Some(existing) = existing_global {
            let mut active: click_stats_global_daily::ActiveModel = existing.into();
            active.total_clicks = Set(total_clicks);
            active.unique_links = Set(Some(unique_links));
            active.top_referrers = Set(Some(top_referrers_json));
            active.top_countries = Set(Some(top_countries_json));
            active.top_sources = Set(Some(top_sources_json));

            retry::with_retry("rollup_update_global_daily", self.retry_config, || async {
                click_stats_global_daily::Entity::update(active.clone())
                    .exec(db)
                    .await
            })
            .await?;
        } else {
            let model = click_stats_global_daily::ActiveModel {
                day_bucket: Set(target_date),
                total_clicks: Set(total_clicks),
                unique_links: Set(Some(unique_links)),
                top_referrers: Set(Some(top_referrers_json)),
                top_countries: Set(Some(top_countries_json)),
                top_sources: Set(Some(top_sources_json)),
                ..Default::default()
            };

            retry::with_retry("rollup_insert_global_daily", self.retry_config, || async {
                click_stats_global_daily::Entity::insert(model.clone())
                    .exec(db)
                    .await
            })
            .await?;
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

    /// 使用 CASE WHEN 批量更新 daily 记录
    async fn batch_update_daily(
        &self,
        db: &impl ConnectionTrait,
        records: &[click_stats_daily::ActiveModel],
    ) -> Result<(), sea_orm::DbErr> {
        if records.is_empty() {
            return Ok(());
        }

        // 一次循环同时收集 ids 和构建 CASE WHEN
        let mut ids: Vec<i64> = Vec::with_capacity(records.len());
        let mut click_count_case = CaseStatement::new();
        let mut unique_referrers_case = CaseStatement::new();
        let mut unique_countries_case = CaseStatement::new();
        let mut unique_sources_case = CaseStatement::new();
        let mut top_referrers_case = CaseStatement::new();
        let mut top_countries_case = CaseStatement::new();
        let mut top_sources_case = CaseStatement::new();

        for record in records {
            if let Set(id) = &record.id {
                ids.push(*id);

                // 使用 ColumnTrait.eq() 返回 SimpleExpr，可直接 clone
                let id_cond = click_stats_daily::Column::Id.eq(*id);

                if let Set(click_count) = &record.click_count {
                    click_count_case = click_count_case
                        .case(id_cond.clone(), SimpleExpr::Value((*click_count).into()));
                }
                if let Set(Some(ur)) = &record.unique_referrers {
                    unique_referrers_case = unique_referrers_case
                        .case(id_cond.clone(), SimpleExpr::Value((*ur).into()));
                }
                if let Set(Some(uc)) = &record.unique_countries {
                    unique_countries_case = unique_countries_case
                        .case(id_cond.clone(), SimpleExpr::Value((*uc).into()));
                }
                if let Set(Some(us)) = &record.unique_sources {
                    unique_sources_case =
                        unique_sources_case.case(id_cond.clone(), SimpleExpr::Value((*us).into()));
                }
                if let Set(Some(tr)) = &record.top_referrers {
                    top_referrers_case = top_referrers_case
                        .case(id_cond.clone(), SimpleExpr::Value(tr.clone().into()));
                }
                if let Set(Some(tc)) = &record.top_countries {
                    top_countries_case = top_countries_case
                        .case(id_cond.clone(), SimpleExpr::Value(tc.clone().into()));
                }
                if let Set(Some(ts)) = &record.top_sources {
                    top_sources_case =
                        top_sources_case.case(id_cond, SimpleExpr::Value(ts.clone().into()));
                }
            }
        }

        // 添加默认值（保持原值）
        click_count_case =
            click_count_case.finally(Expr::col(click_stats_daily::Column::ClickCount));
        unique_referrers_case =
            unique_referrers_case.finally(Expr::col(click_stats_daily::Column::UniqueReferrers));
        unique_countries_case =
            unique_countries_case.finally(Expr::col(click_stats_daily::Column::UniqueCountries));
        unique_sources_case =
            unique_sources_case.finally(Expr::col(click_stats_daily::Column::UniqueSources));
        top_referrers_case =
            top_referrers_case.finally(Expr::col(click_stats_daily::Column::TopReferrers));
        top_countries_case =
            top_countries_case.finally(Expr::col(click_stats_daily::Column::TopCountries));
        top_sources_case =
            top_sources_case.finally(Expr::col(click_stats_daily::Column::TopSources));

        // 使用 SeaORM 官方 update_many API
        click_stats_daily::Entity::update_many()
            .col_expr(
                click_stats_daily::Column::ClickCount,
                click_count_case.into(),
            )
            .col_expr(
                click_stats_daily::Column::UniqueReferrers,
                unique_referrers_case.into(),
            )
            .col_expr(
                click_stats_daily::Column::UniqueCountries,
                unique_countries_case.into(),
            )
            .col_expr(
                click_stats_daily::Column::UniqueSources,
                unique_sources_case.into(),
            )
            .col_expr(
                click_stats_daily::Column::TopReferrers,
                top_referrers_case.into(),
            )
            .col_expr(
                click_stats_daily::Column::TopCountries,
                top_countries_case.into(),
            )
            .col_expr(
                click_stats_daily::Column::TopSources,
                top_sources_case.into(),
            )
            .filter(click_stats_daily::Column::Id.is_in(ids))
            .exec(db)
            .await?;

        Ok(())
    }

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

        // 聚合 source
        if let Some(ref source) = detail.source {
            *agg.sources.entry(source.clone()).or_insert(0) += 1;
        } else {
            *agg.sources.entry("direct".to_string()).or_insert(0) += 1;
        }
    }

    result
}
