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
use sea_orm::{ActiveValue::Set, ConnectionTrait, EntityTrait, ExprTrait};
use tracing::{debug, warn};

use super::SeaOrmStorage;
use super::retry;
use crate::analytics::{
    ClickDetail, ClickSink, DetailedClickSink, HourlyRollupWriter, truncate_to_hour,
};
use crate::utils::is_valid_short_code;

use migration::entities::{click_log, short_link};

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
        .map_err(|e| {
            anyhow::anyhow!(
                "Failed to batch update click counts (still failed after retries): {}",
                e
            )
        })?;

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
                user_agent_hash: Set(detail.user_agent_hash.clone()),
                ip_address: Set(detail.ip_address.clone()),
                country: Set(detail.country.clone()),
                city: Set(detail.city.clone()),
                source: Set(detail.source.clone()),
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
        .map_err(|e| anyhow::anyhow!("Failed to batch insert click logs: {}", e))?;

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

impl SeaOrmStorage {
    /// 创建 HourlyRollupWriter 实例
    fn hourly_writer(&self) -> HourlyRollupWriter<'_, impl ConnectionTrait> {
        HourlyRollupWriter::new(&self.db, self.retry_config)
    }

    /// 更新小时汇总（仅计数）
    async fn update_hourly_rollup(
        &self,
        updates: &[(String, usize)],
        timestamp: DateTime<Utc>,
    ) -> anyhow::Result<()> {
        let hour_bucket = truncate_to_hour(timestamp);
        let writer = self.hourly_writer();

        // 更新各链接的小时汇总
        writer
            .upsert_hourly_counts(updates, timestamp, "sink")
            .await?;

        // 更新全局小时汇总
        let total_clicks: usize = updates.iter().map(|(_, c)| c).sum();
        let unique_links = updates.len() as i32;
        writer
            .upsert_global_hourly(hour_bucket, total_clicks, unique_links, "sink")
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

        // 复用已有的聚合函数
        let aggregated = crate::analytics::aggregate_click_details(details);

        self.hourly_writer()
            .upsert_hourly_with_details(&aggregated, "sink")
            .await
    }
}
