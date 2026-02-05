//! ClickSink implementation for SeaOrmStorage
//!
//! This module implements the click tracking flush functionality.
//!
//! # Security Note
//!
//! This module uses parameterized queries via `DatabaseBackend::build()` for SQL safety.
//! All `short_code` values are additionally validated via `utils::is_valid_short_code()`
//! as defense-in-depth against SQL injection.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::sea_query::{CaseStatement, Expr, Query};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, ExprTrait, QueryFilter,
};
use std::collections::HashMap;
use tracing::{debug, warn};

use super::SeaOrmStorage;
use super::retry;
use crate::analytics::{ClickDetail, ClickSink, DetailedClickSink};
use crate::utils::is_valid_short_code;

use migration::entities::{click_log, click_stats_global_hourly, click_stats_hourly, short_link};

#[async_trait]
impl ClickSink for SeaOrmStorage {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
        if updates.is_empty() {
            return Ok(());
        }

        // 安全校验：确保所有 short_code 格式合法，防止 SQL 注入
        for (code, _) in &updates {
            if !is_valid_short_code(code) {
                return Err(anyhow::anyhow!(
                    "Invalid short_code format detected: '{}' - refusing to execute SQL",
                    code
                ));
            }
        }

        let total_count = updates.len();

        // 构建 CASE WHEN 表达式（跨平台兼容）
        let mut case_stmt = CaseStatement::new();
        let mut codes: Vec<String> = Vec::with_capacity(total_count);

        for (code, count) in &updates {
            case_stmt = case_stmt.case(
                Expr::col(short_link::Column::ShortCode).eq(Expr::val(code.as_str())),
                Expr::col(short_link::Column::ClickCount).add(Expr::val(*count as i64)),
            );
            codes.push(code.clone());
        }
        // 不匹配的保持原值
        case_stmt = case_stmt.finally(Expr::col(short_link::Column::ClickCount));

        // 构建 UPDATE 语句
        let stmt = Query::update()
            .table(short_link::Entity)
            .value(short_link::Column::ClickCount, case_stmt)
            .and_where(Expr::col(short_link::Column::ShortCode).is_in(codes))
            .to_owned();

        // 使用参数化查询执行（SeaORM 内部自动 build 为带绑定参数的 Statement）
        let db = &self.db;
        let stmt_ref = &stmt;
        retry::with_retry("flush_clicks", self.retry_config, || async {
            db.execute(stmt_ref).await
        })
        .await
        .map_err(|e| anyhow::anyhow!("批量更新点击数失败（重试后仍失败）: {}", e))?;

        debug!(
            "Click counts flushed to {} database ({} records)",
            self.backend_name.to_uppercase(),
            total_count
        );

        // 同步更新小时汇总表
        if let Err(e) = self.update_hourly_rollup(&updates, Utc::now()).await {
            warn!("Failed to update hourly rollup (non-blocking): {}", e);
        }

        Ok(())
    }
}

#[async_trait]
impl DetailedClickSink for SeaOrmStorage {
    async fn log_click(&self, detail: ClickDetail) -> anyhow::Result<()> {
        self.log_clicks_batch(vec![detail]).await
    }

    async fn log_clicks_batch(&self, details: Vec<ClickDetail>) -> anyhow::Result<()> {
        if details.is_empty() {
            return Ok(());
        }

        let total_count = details.len();

        // 构建批量插入的 ActiveModel 列表
        let models: Vec<click_log::ActiveModel> = details
            .iter()
            .map(|detail| click_log::ActiveModel {
                short_code: Set(detail.code.clone()),
                clicked_at: Set(detail.timestamp),
                referrer: Set(detail.referrer.clone()),
                user_agent: Set(detail.user_agent.clone()),
                user_agent_hash: Set(detail.user_agent_hash.clone()),
                ip_address: Set(detail.ip_address.clone()),
                country: Set(detail.country.clone()),
                city: Set(detail.city.clone()),
                ..Default::default()
            })
            .collect();

        // 使用 insert_many 进行批量插入
        let db = &self.db;
        retry::with_retry("log_clicks_batch", self.retry_config, || async {
            click_log::Entity::insert_many(models.clone())
                .exec(db)
                .await
        })
        .await
        .map_err(|e| anyhow::anyhow!("批量插入点击日志失败: {}", e))?;

        debug!(
            "Detailed click logs written to {} database ({} records)",
            self.backend_name.to_uppercase(),
            total_count
        );

        // 更新带详细信息的小时汇总
        if let Err(e) = self.update_hourly_rollup_with_details(&details).await {
            warn!(
                "Failed to update detailed hourly rollup (non-blocking): {}",
                e
            );
        }

        Ok(())
    }
}

/// 小时级聚合值: (count, referrer_counts, country_counts)
type HourlyAggregation = (usize, HashMap<String, usize>, HashMap<String, usize>);

impl SeaOrmStorage {
    /// 更新小时汇总（仅计数）
    async fn update_hourly_rollup(
        &self,
        updates: &[(String, usize)],
        timestamp: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let hour_bucket = crate::analytics::truncate_to_hour(timestamp);
        let db = &self.db;

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
                retry::with_retry("sink_update_hourly", self.retry_config, || async {
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
                retry::with_retry("sink_insert_hourly", self.retry_config, || async {
                    click_stats_hourly::Entity::insert(model.clone())
                        .exec(db)
                        .await
                })
                .await?;
            }
        }

        // 更新全局小时汇总
        let total_clicks: usize = updates.iter().map(|(_, c)| c).sum();
        let unique_links = updates.len() as i32;
        self.update_global_hourly(hour_bucket, total_clicks, unique_links)
            .await?;

        debug!(
            "Hourly rollup updated: {} links (bucket: {})",
            updates.len(),
            hour_bucket
        );

        Ok(())
    }

    /// 更新小时汇总（含详细信息）
    async fn update_hourly_rollup_with_details(
        &self,
        details: &[ClickDetail],
    ) -> anyhow::Result<()> {
        if details.is_empty() {
            return Ok(());
        }

        let db = &self.db;

        // 按 (short_code, hour_bucket) 聚合
        let mut aggregated: HashMap<(String, DateTime<Utc>), HourlyAggregation> = HashMap::new();

        for detail in details {
            let hour_bucket = crate::analytics::truncate_to_hour(detail.timestamp);
            let key = (detail.code.clone(), hour_bucket);

            let entry = aggregated
                .entry(key)
                .or_insert_with(|| (0, HashMap::new(), HashMap::new()));
            entry.0 += 1;

            // 统计 referrer
            let referrer_key = detail
                .referrer
                .clone()
                .unwrap_or_else(|| "direct".to_string());
            *entry.1.entry(referrer_key).or_insert(0) += 1;

            // 统计 country
            let country_key = detail
                .country
                .clone()
                .unwrap_or_else(|| "Unknown".to_string());
            *entry.2.entry(country_key).or_insert(0) += 1;
        }

        for ((code, hour_bucket), (count, referrers, countries)) in aggregated {
            let existing = click_stats_hourly::Entity::find()
                .filter(click_stats_hourly::Column::ShortCode.eq(&code))
                .filter(click_stats_hourly::Column::HourBucket.eq(hour_bucket))
                .one(db)
                .await?;

            if let Some(record) = existing {
                // 合并现有数据
                let mut merged_referrers =
                    crate::analytics::parse_json_counts(&record.referrer_counts);
                let mut merged_countries =
                    crate::analytics::parse_json_counts(&record.country_counts);

                for (k, v) in &referrers {
                    *merged_referrers.entry(k.clone()).or_insert(0) += v;
                }
                for (k, v) in &countries {
                    *merged_countries.entry(k.clone()).or_insert(0) += v;
                }

                let mut active: click_stats_hourly::ActiveModel = record.into();
                if let Set(old_count) = active.click_count {
                    active.click_count = Set(old_count + count as i64);
                }
                active.referrer_counts =
                    Set(Some(crate::analytics::to_json_string(&merged_referrers)));
                active.country_counts =
                    Set(Some(crate::analytics::to_json_string(&merged_countries)));

                retry::with_retry("sink_update_hourly_detailed", self.retry_config, || async {
                    click_stats_hourly::Entity::update(active.clone())
                        .exec(db)
                        .await
                })
                .await?;
            } else {
                let model = click_stats_hourly::ActiveModel {
                    short_code: Set(code),
                    hour_bucket: Set(hour_bucket),
                    click_count: Set(count as i64),
                    referrer_counts: Set(Some(crate::analytics::to_json_string(&referrers))),
                    country_counts: Set(Some(crate::analytics::to_json_string(&countries))),
                    ..Default::default()
                };
                retry::with_retry("sink_insert_hourly_detailed", self.retry_config, || async {
                    click_stats_hourly::Entity::insert(model.clone())
                        .exec(db)
                        .await
                })
                .await?;
            }
        }

        Ok(())
    }

    /// 更新全局小时汇总
    async fn update_global_hourly(
        &self,
        hour_bucket: DateTime<Utc>,
        clicks: usize,
        unique_links: i32,
    ) -> anyhow::Result<()> {
        let db = &self.db;

        let existing = click_stats_global_hourly::Entity::find()
            .filter(click_stats_global_hourly::Column::HourBucket.eq(hour_bucket))
            .one(db)
            .await?;

        if let Some(record) = existing {
            let mut active: click_stats_global_hourly::ActiveModel = record.into();
            if let Set(old_clicks) = active.total_clicks {
                active.total_clicks = Set(old_clicks + clicks as i64);
            }
            // unique_links 不做累加（会导致重复计数），留给 rollup 从 hourly 表统计
            retry::with_retry("sink_update_global_hourly", self.retry_config, || async {
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
            retry::with_retry("sink_insert_global_hourly", self.retry_config, || async {
                click_stats_global_hourly::Entity::insert(model.clone())
                    .exec(db)
                    .await
            })
            .await?;
        }

        Ok(())
    }
}
