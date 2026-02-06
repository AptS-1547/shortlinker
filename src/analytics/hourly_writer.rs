//! 小时汇总写入器
//!
//! 统一 click_sink 和 rollup 的汇总逻辑，避免代码重复。
//! 参见 GitHub Issue #76

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sea_orm::{ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use tracing::debug;

use crate::storage::backend::retry::{self, RetryConfig};
use migration::entities::{click_stats_global_hourly, click_stats_hourly};

use super::{ClickAggregation, parse_json_counts, to_json_string, truncate_to_hour};

/// 小时汇总写入器
///
/// 封装小时汇总表的 upsert 逻辑，供 ClickSink 和 RollupManager 共用。
pub struct HourlyRollupWriter<'a, C: ConnectionTrait> {
    db: &'a C,
    retry_config: RetryConfig,
}

impl<'a, C: ConnectionTrait> HourlyRollupWriter<'a, C> {
    pub fn new(db: &'a C, retry_config: RetryConfig) -> Self {
        Self { db, retry_config }
    }

    /// 更新小时汇总（仅计数，无详细信息）
    ///
    /// # Arguments
    /// - `updates`: (short_code, click_count) 列表
    /// - `timestamp`: 时间戳，用于确定 hour_bucket
    /// - `op_prefix`: 操作名前缀，用于重试日志（如 "sink" 或 "rollup"）
    pub async fn upsert_hourly_counts(
        &self,
        updates: &[(String, usize)],
        timestamp: DateTime<Utc>,
        op_prefix: &str,
    ) -> anyhow::Result<()> {
        if updates.is_empty() {
            return Ok(());
        }

        let hour_bucket = truncate_to_hour(timestamp);

        for (code, count) in updates {
            let existing = click_stats_hourly::Entity::find()
                .filter(click_stats_hourly::Column::ShortCode.eq(code.as_str()))
                .filter(click_stats_hourly::Column::HourBucket.eq(hour_bucket))
                .one(self.db)
                .await?;

            if let Some(record) = existing {
                let mut active: click_stats_hourly::ActiveModel = record.into();
                if let Set(old_count) = active.click_count {
                    active.click_count = Set(old_count + *count as i64);
                }
                let op_name = format!("{}_update_hourly", op_prefix);
                retry::with_retry(&op_name, self.retry_config, || async {
                    click_stats_hourly::Entity::update(active.clone())
                        .exec(self.db)
                        .await
                })
                .await?;
            } else {
                let model = click_stats_hourly::ActiveModel {
                    short_code: Set(code.clone()),
                    hour_bucket: Set(hour_bucket),
                    click_count: Set(*count as i64),
                    referrer_counts: Set(None),
                    country_counts: Set(None),
                    source_counts: Set(None),
                    ..Default::default()
                };
                let op_name = format!("{}_insert_hourly", op_prefix);
                retry::with_retry(&op_name, self.retry_config, || async {
                    click_stats_hourly::Entity::insert(model.clone())
                        .exec(self.db)
                        .await
                })
                .await?;
            }
        }

        debug!(
            "[{}] Hourly counts updated: {} links (bucket: {})",
            op_prefix,
            updates.len(),
            hour_bucket
        );

        Ok(())
    }

    /// 更新小时汇总（含详细信息）
    ///
    /// # Arguments
    /// - `aggregated`: 已聚合的点击数据，key 为 (short_code, hour_bucket)
    /// - `op_prefix`: 操作名前缀
    pub async fn upsert_hourly_with_details(
        &self,
        aggregated: &HashMap<(String, DateTime<Utc>), ClickAggregation>,
        op_prefix: &str,
    ) -> anyhow::Result<()> {
        if aggregated.is_empty() {
            return Ok(());
        }

        for ((code, hour_bucket), agg) in aggregated {
            let existing = click_stats_hourly::Entity::find()
                .filter(click_stats_hourly::Column::ShortCode.eq(code.as_str()))
                .filter(click_stats_hourly::Column::HourBucket.eq(*hour_bucket))
                .one(self.db)
                .await?;

            if let Some(record) = existing {
                // 合并现有数据
                let mut merged_referrers = parse_json_counts(&record.referrer_counts);
                let mut merged_countries = parse_json_counts(&record.country_counts);
                let mut merged_sources = parse_json_counts(&record.source_counts);

                for (k, v) in &agg.referrers {
                    *merged_referrers.entry(k.clone()).or_insert(0) += v;
                }
                for (k, v) in &agg.countries {
                    *merged_countries.entry(k.clone()).or_insert(0) += v;
                }
                for (k, v) in &agg.sources {
                    *merged_sources.entry(k.clone()).or_insert(0) += v;
                }

                let mut active: click_stats_hourly::ActiveModel = record.into();
                if let Set(old_count) = active.click_count {
                    active.click_count = Set(old_count + agg.count as i64);
                }
                active.referrer_counts = Set(Some(to_json_string(&merged_referrers)));
                active.country_counts = Set(Some(to_json_string(&merged_countries)));
                active.source_counts = Set(Some(to_json_string(&merged_sources)));

                let op_name = format!("{}_update_hourly_detailed", op_prefix);
                retry::with_retry(&op_name, self.retry_config, || async {
                    click_stats_hourly::Entity::update(active.clone())
                        .exec(self.db)
                        .await
                })
                .await?;
            } else {
                let model = click_stats_hourly::ActiveModel {
                    short_code: Set(code.clone()),
                    hour_bucket: Set(*hour_bucket),
                    click_count: Set(agg.count as i64),
                    referrer_counts: Set(Some(to_json_string(&agg.referrers))),
                    country_counts: Set(Some(to_json_string(&agg.countries))),
                    source_counts: Set(Some(to_json_string(&agg.sources))),
                    ..Default::default()
                };
                let op_name = format!("{}_insert_hourly_detailed", op_prefix);
                retry::with_retry(&op_name, self.retry_config, || async {
                    click_stats_hourly::Entity::insert(model.clone())
                        .exec(self.db)
                        .await
                })
                .await?;
            }
        }

        debug!(
            "[{}] Detailed hourly rollup updated: {} records",
            op_prefix,
            aggregated.len()
        );

        Ok(())
    }

    /// 更新全局小时汇总
    ///
    /// # Arguments
    /// - `hour_bucket`: 小时时间桶
    /// - `clicks`: 点击数
    /// - `unique_links`: 唯一链接数
    /// - `op_prefix`: 操作名前缀
    pub async fn upsert_global_hourly(
        &self,
        hour_bucket: DateTime<Utc>,
        clicks: usize,
        unique_links: i32,
        op_prefix: &str,
    ) -> anyhow::Result<()> {
        let existing = click_stats_global_hourly::Entity::find()
            .filter(click_stats_global_hourly::Column::HourBucket.eq(hour_bucket))
            .one(self.db)
            .await?;

        if let Some(record) = existing {
            let mut active: click_stats_global_hourly::ActiveModel = record.into();
            if let Set(old_clicks) = active.total_clicks {
                active.total_clicks = Set(old_clicks + clicks as i64);
            }
            // unique_links 不做累加（会导致重复计数）
            let op_name = format!("{}_update_global_hourly", op_prefix);
            retry::with_retry(&op_name, self.retry_config, || async {
                click_stats_global_hourly::Entity::update(active.clone())
                    .exec(self.db)
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
            let op_name = format!("{}_insert_global_hourly", op_prefix);
            retry::with_retry(&op_name, self.retry_config, || async {
                click_stats_global_hourly::Entity::insert(model.clone())
                    .exec(self.db)
                    .await
            })
            .await?;
        }

        Ok(())
    }
}
