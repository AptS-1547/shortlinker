//! 小时汇总写入器
//!
//! 统一 click_sink 和 rollup 的汇总逻辑，避免代码重复。
//! 参见 GitHub Issue #76
//!
//! ## 性能优化
//!
//! 使用批量 upsert 替代逐条查询+更新，将 O(2N) 数据库操作降为 O(1)。

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sea_orm::{
    ActiveValue::Set,
    ConnectionTrait, DatabaseBackend, EntityTrait, ExprTrait,
    sea_query::{Expr, OnConflict, Query},
};
use tracing::debug;

use crate::storage::backend::retry::{self, RetryConfig};
use migration::entities::{click_stats_global_hourly, click_stats_hourly};

use super::{ClickAggregation, to_json_string, truncate_to_hour};

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
    /// 使用批量 upsert 实现，单条 SQL 处理所有记录。
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
        let backend = self.db.get_database_backend();

        // 批量 upsert：单条 SQL 处理所有记录
        let op_name = format!("{}_upsert_hourly_counts", op_prefix);
        retry::with_retry(&op_name, self.retry_config, || async {
            self.batch_upsert_counts(updates, hour_bucket, backend)
                .await
        })
        .await?;

        debug!(
            "[{}] Hourly counts updated: {} links (bucket: {})",
            op_prefix,
            updates.len(),
            hour_bucket
        );

        Ok(())
    }

    /// 执行批量 upsert（仅计数）
    async fn batch_upsert_counts(
        &self,
        updates: &[(String, usize)],
        hour_bucket: DateTime<Utc>,
        backend: DatabaseBackend,
    ) -> Result<(), sea_orm::DbErr> {
        // 构建批量插入的 ActiveModel 列表
        let models: Vec<click_stats_hourly::ActiveModel> = updates
            .iter()
            .map(|(code, count)| click_stats_hourly::ActiveModel {
                short_code: Set(code.clone()),
                hour_bucket: Set(hour_bucket),
                click_count: Set(*count as i64),
                referrer_counts: Set(None),
                country_counts: Set(None),
                source_counts: Set(None),
                ..Default::default()
            })
            .collect();

        // 使用 SeaORM 的 on_conflict 配合 value() 表达式实现累加
        // SQLite/PostgreSQL: click_count = click_count + excluded.click_count
        // MySQL: click_count = click_count + VALUES(click_count)
        let on_conflict = match backend {
            DatabaseBackend::MySql => {
                // MySQL 语法：VALUES(column)
                OnConflict::columns([
                    click_stats_hourly::Column::ShortCode,
                    click_stats_hourly::Column::HourBucket,
                ])
                .value(
                    click_stats_hourly::Column::ClickCount,
                    Expr::col(click_stats_hourly::Column::ClickCount)
                        .add(Expr::cust("VALUES(click_count)")),
                )
                .to_owned()
            }
            _ => {
                // SQLite/PostgreSQL 语法：excluded.column
                OnConflict::columns([
                    click_stats_hourly::Column::ShortCode,
                    click_stats_hourly::Column::HourBucket,
                ])
                .value(
                    click_stats_hourly::Column::ClickCount,
                    Expr::col(click_stats_hourly::Column::ClickCount)
                        .add(Expr::cust("excluded.click_count")),
                )
                .to_owned()
            }
        };

        click_stats_hourly::Entity::insert_many(models)
            .on_conflict(on_conflict)
            .exec(self.db)
            .await?;

        Ok(())
    }

    /// 更新小时汇总（含详细信息）
    ///
    /// 由于需要合并 JSON 字段，这里仍需逐条处理，但使用 upsert 替代 select+insert/update。
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

        let backend = self.db.get_database_backend();

        // 由于 JSON 字段需要合并，无法用简单的 upsert
        // 使用 raw SQL 批量读取现有记录，在内存中合并后批量更新
        let keys: Vec<_> = aggregated.keys().collect();
        let existing_records = self.batch_fetch_hourly_records(&keys).await?;

        // 分离新增和更新
        let mut to_insert = Vec::new();
        let mut to_update = Vec::new();

        for ((code, hour_bucket), agg) in aggregated {
            let key = (code.clone(), *hour_bucket);
            if let Some(record) = existing_records.get(&key) {
                // 合并现有数据
                let merged = self.merge_aggregation(record, agg);
                to_update.push((record.id, merged));
            } else {
                // 新记录
                to_insert.push((code.clone(), *hour_bucket, agg.clone()));
            }
        }

        // 批量插入新记录
        if !to_insert.is_empty() {
            let op_name = format!("{}_insert_hourly_detailed", op_prefix);
            retry::with_retry(&op_name, self.retry_config, || async {
                self.batch_insert_detailed(&to_insert).await
            })
            .await?;
        }

        // 批量更新现有记录（分批处理，避免超出 SQL 变量限制）
        if !to_update.is_empty() {
            const UPDATE_BATCH_SIZE: usize = 100;
            for chunk in to_update.chunks(UPDATE_BATCH_SIZE) {
                let op_name = format!("{}_update_hourly_detailed", op_prefix);
                let chunk_vec = chunk.to_vec();
                retry::with_retry(&op_name, self.retry_config, || async {
                    self.batch_update_detailed(&chunk_vec, backend).await
                })
                .await?;
            }
        }

        debug!(
            "[{}] Detailed hourly rollup updated: {} records ({} new, {} updated)",
            op_prefix,
            aggregated.len(),
            to_insert.len(),
            to_update.len()
        );

        Ok(())
    }

    /// 批量获取现有的小时汇总记录
    async fn batch_fetch_hourly_records(
        &self,
        keys: &[&(String, DateTime<Utc>)],
    ) -> anyhow::Result<HashMap<(String, DateTime<Utc>), click_stats_hourly::Model>> {
        use sea_orm::{ColumnTrait, QueryFilter};

        if keys.is_empty() {
            return Ok(HashMap::new());
        }

        // 提取所有 hour_bucket（通常只有一个）
        let hour_buckets: Vec<_> = keys
            .iter()
            .map(|(_, h)| *h)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        let codes: Vec<_> = keys.iter().map(|(c, _)| c.as_str()).collect();

        let records = click_stats_hourly::Entity::find()
            .filter(click_stats_hourly::Column::ShortCode.is_in(codes))
            .filter(click_stats_hourly::Column::HourBucket.is_in(hour_buckets))
            .all(self.db)
            .await?;

        let result: HashMap<_, _> = records
            .into_iter()
            .map(|r| ((r.short_code.clone(), r.hour_bucket), r))
            .collect();

        Ok(result)
    }

    /// 合并 aggregation 数据
    fn merge_aggregation(
        &self,
        record: &click_stats_hourly::Model,
        agg: &ClickAggregation,
    ) -> ClickAggregation {
        use super::parse_json_counts;

        let mut merged = agg.clone();

        // 合并计数
        merged.count += record.click_count as usize;

        // 合并 referrers
        let existing_referrers = parse_json_counts(&record.referrer_counts);
        for (k, v) in existing_referrers {
            *merged.referrers.entry(k).or_insert(0) += v;
        }

        // 合并 countries
        let existing_countries = parse_json_counts(&record.country_counts);
        for (k, v) in existing_countries {
            *merged.countries.entry(k).or_insert(0) += v;
        }

        // 合并 sources
        let existing_sources = parse_json_counts(&record.source_counts);
        for (k, v) in existing_sources {
            *merged.sources.entry(k).or_insert(0) += v;
        }

        merged
    }

    /// 批量插入详细记录
    async fn batch_insert_detailed(
        &self,
        records: &[(String, DateTime<Utc>, ClickAggregation)],
    ) -> Result<(), sea_orm::DbErr> {
        if records.is_empty() {
            return Ok(());
        }

        let models: Vec<click_stats_hourly::ActiveModel> = records
            .iter()
            .map(|(code, hour_bucket, agg)| click_stats_hourly::ActiveModel {
                short_code: Set(code.clone()),
                hour_bucket: Set(*hour_bucket),
                click_count: Set(agg.count as i64),
                referrer_counts: Set(Some(to_json_string(&agg.referrers))),
                country_counts: Set(Some(to_json_string(&agg.countries))),
                source_counts: Set(Some(to_json_string(&agg.sources))),
                ..Default::default()
            })
            .collect();

        click_stats_hourly::Entity::insert_many(models)
            .exec(self.db)
            .await?;

        Ok(())
    }

    /// 批量更新详细记录
    async fn batch_update_detailed(
        &self,
        records: &[(i64, ClickAggregation)],
        _backend: DatabaseBackend,
    ) -> Result<(), sea_orm::DbErr> {
        if records.is_empty() {
            return Ok(());
        }

        // 使用 CASE WHEN 批量更新，类似 click_sink.rs 的实现
        use sea_orm::sea_query::{CaseStatement, SimpleExpr};

        let ids: Vec<i64> = records.iter().map(|(id, _)| *id).collect();

        // 构建每个字段的 CASE WHEN
        let mut click_count_case = CaseStatement::new();
        let mut referrer_case = CaseStatement::new();
        let mut country_case = CaseStatement::new();
        let mut source_case = CaseStatement::new();

        for (id, agg) in records {
            let id_expr = Expr::col(click_stats_hourly::Column::Id).eq(Expr::val(*id));

            click_count_case = click_count_case.case(
                id_expr.clone(),
                SimpleExpr::Value((agg.count as i64).into()),
            );
            referrer_case = referrer_case.case(
                id_expr.clone(),
                SimpleExpr::Value(to_json_string(&agg.referrers).into()),
            );
            country_case = country_case.case(
                id_expr.clone(),
                SimpleExpr::Value(to_json_string(&agg.countries).into()),
            );
            source_case = source_case.case(
                id_expr,
                SimpleExpr::Value(to_json_string(&agg.sources).into()),
            );
        }

        // 不匹配的保持原值
        click_count_case =
            click_count_case.finally(Expr::col(click_stats_hourly::Column::ClickCount));
        referrer_case =
            referrer_case.finally(Expr::col(click_stats_hourly::Column::ReferrerCounts));
        country_case = country_case.finally(Expr::col(click_stats_hourly::Column::CountryCounts));
        source_case = source_case.finally(Expr::col(click_stats_hourly::Column::SourceCounts));

        let stmt = Query::update()
            .table(click_stats_hourly::Entity)
            .value(click_stats_hourly::Column::ClickCount, click_count_case)
            .value(click_stats_hourly::Column::ReferrerCounts, referrer_case)
            .value(click_stats_hourly::Column::CountryCounts, country_case)
            .value(click_stats_hourly::Column::SourceCounts, source_case)
            .and_where(Expr::col(click_stats_hourly::Column::Id).is_in(ids))
            .to_owned();

        self.db.execute(&stmt).await?;

        Ok(())
    }

    /// 更新全局小时汇总
    ///
    /// 使用 upsert 替代 select+insert/update。
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
        let backend = self.db.get_database_backend();

        let model = click_stats_global_hourly::ActiveModel {
            hour_bucket: Set(hour_bucket),
            total_clicks: Set(clicks as i64),
            unique_links: Set(Some(unique_links)),
            top_referrers: Set(None),
            top_countries: Set(None),
            ..Default::default()
        };

        // 构建 upsert：total_clicks 累加，unique_links 不累加（取最新值）
        let on_conflict = match backend {
            DatabaseBackend::MySql => {
                OnConflict::column(click_stats_global_hourly::Column::HourBucket)
                    .value(
                        click_stats_global_hourly::Column::TotalClicks,
                        Expr::col(click_stats_global_hourly::Column::TotalClicks)
                            .add(Expr::cust("VALUES(total_clicks)")),
                    )
                    .to_owned()
            }
            _ => OnConflict::column(click_stats_global_hourly::Column::HourBucket)
                .value(
                    click_stats_global_hourly::Column::TotalClicks,
                    Expr::col(click_stats_global_hourly::Column::TotalClicks)
                        .add(Expr::cust("excluded.total_clicks")),
                )
                .to_owned(),
        };

        let op_name = format!("{}_upsert_global_hourly", op_prefix);
        retry::with_retry(&op_name, self.retry_config, || async {
            click_stats_global_hourly::Entity::insert(model.clone())
                .on_conflict(on_conflict.clone())
                .exec(self.db)
                .await
        })
        .await?;

        Ok(())
    }
}
