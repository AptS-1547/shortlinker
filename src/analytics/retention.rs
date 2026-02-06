//! 数据清理任务
//!
//! 负责清理过期的点击日志和汇总数据，防止数据库无限增长。

use std::sync::Arc;
use std::time::Duration as StdDuration;

use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use tracing::{debug, error, info, warn};

use crate::analytics::global::set_detailed_logging_stopped;
use crate::config::runtime_config::get_runtime_config;
use crate::storage::backend::SeaOrmStorage;
use migration::entities::click_log;

use super::RollupManager;

/// 清理报告
#[derive(Debug, Default)]
pub struct CleanupReport {
    /// 删除的原始点击日志数量
    pub raw_logs_deleted: u64,
    /// 因行数限制删除的日志数量
    pub rows_limit_deleted: u64,
    /// 删除的小时汇总数量
    pub hourly_stats_deleted: u64,
    /// 删除的天汇总数量
    pub daily_stats_deleted: u64,
}

/// 数据清理任务
pub struct DataRetentionTask {
    storage: Arc<SeaOrmStorage>,
    rollup_manager: Arc<RollupManager>,
    /// 原始点击日志保留天数
    raw_log_retention_days: u64,
    /// 小时汇总保留天数
    hourly_retention_days: u64,
    /// 天汇总保留天数
    daily_retention_days: u64,
    /// 每次删除批量大小
    batch_size: u64,
    /// 最大日志行数（0 = 不限制）
    max_log_rows: u64,
    /// 超过最大行数时的动作
    max_rows_action: String,
}

impl DataRetentionTask {
    pub fn new(storage: Arc<SeaOrmStorage>, rollup_manager: Arc<RollupManager>) -> Self {
        // 从运行时配置读取保留天数
        let runtime_config = get_runtime_config();

        let raw_log_retention_days = runtime_config.get_u64_or("analytics.log_retention_days", 30);

        let hourly_retention_days = runtime_config.get_u64_or("analytics.hourly_retention_days", 7);

        let daily_retention_days = runtime_config.get_u64_or("analytics.daily_retention_days", 365);

        let max_log_rows = runtime_config.get_u64_or("analytics.max_log_rows", 0);

        let max_rows_action = runtime_config.get_or("analytics.max_rows_action", "cleanup");

        Self {
            storage,
            rollup_manager,
            raw_log_retention_days,
            hourly_retention_days,
            daily_retention_days,
            batch_size: 10000,
            max_log_rows,
            max_rows_action,
        }
    }

    /// 检查是否超过最大行数限制，返回 true 表示应该停止记录
    pub fn should_stop_logging(&self) -> bool {
        self.max_log_rows > 0 && self.max_rows_action == "stop"
    }

    /// 获取最大行数限制
    pub fn get_max_log_rows(&self) -> u64 {
        self.max_log_rows
    }

    /// 运行完整的清理流程
    pub async fn run_cleanup(&self) -> anyhow::Result<CleanupReport> {
        let mut report = CleanupReport::default();

        // 1. 清理原始点击日志（按时间）
        match self.cleanup_raw_logs().await {
            Ok(deleted) => {
                report.raw_logs_deleted = deleted;
            }
            Err(e) => {
                error!("Failed to clean up raw click logs: {}", e);
            }
        }

        // 2. 按行数限制处理
        if self.max_log_rows > 0 {
            if self.max_rows_action == "cleanup" {
                // cleanup 模式：删除最旧的记录
                match self.cleanup_by_row_limit().await {
                    Ok(deleted) => {
                        report.rows_limit_deleted = deleted;
                    }
                    Err(e) => {
                        error!("Failed to clean up logs by row limit: {}", e);
                    }
                }
            } else if self.max_rows_action == "stop" {
                // stop 模式：检查行数并设置停止标志
                match self.check_and_update_stop_flag().await {
                    Ok(stopped) => {
                        if stopped {
                            info!("Detailed logging stopped due to max_log_rows limit");
                        }
                    }
                    Err(e) => {
                        error!("Failed to check row limit: {}", e);
                    }
                }
            }
        }

        // 3. 清理汇总数据
        match self
            .rollup_manager
            .cleanup_expired(self.hourly_retention_days, self.daily_retention_days)
            .await
        {
            Ok((hourly, daily)) => {
                report.hourly_stats_deleted = hourly;
                report.daily_stats_deleted = daily;
            }
            Err(e) => {
                error!("Failed to clean up rollup data: {}", e);
            }
        }

        info!(
            "Data cleanup completed: raw logs {} (time-based), {} (row-limit), hourly rollups {}, daily rollups {}",
            report.raw_logs_deleted,
            report.rows_limit_deleted,
            report.hourly_stats_deleted,
            report.daily_stats_deleted
        );

        Ok(report)
    }

    /// 按行数限制清理（删除最旧的记录直到满足限制）
    async fn cleanup_by_row_limit(&self) -> anyhow::Result<u64> {
        let db = self.storage.get_db();

        // 获取当前行数
        let current_count: i64 = click_log::Entity::find()
            .select_only()
            .column_as(click_log::Column::Id.count(), "count")
            .into_tuple()
            .one(db)
            .await?
            .unwrap_or(0);

        if current_count as u64 <= self.max_log_rows {
            debug!(
                "Click logs count {} is within limit {}",
                current_count, self.max_log_rows
            );
            return Ok(0);
        }

        let rows_to_delete = current_count as u64 - self.max_log_rows;
        info!(
            "Click logs count {} exceeds limit {}, need to delete {} rows",
            current_count, self.max_log_rows, rows_to_delete
        );

        let mut total_deleted = 0u64;
        let mut remaining = rows_to_delete;

        while remaining > 0 {
            let batch = remaining.min(self.batch_size);

            // 查找最旧的 N 条记录的 ID
            let ids_to_delete: Vec<i64> = click_log::Entity::find()
                .select_only()
                .column(click_log::Column::Id)
                .order_by_asc(click_log::Column::Id)
                .limit(batch)
                .into_tuple()
                .all(db)
                .await?;

            if ids_to_delete.is_empty() {
                break;
            }

            let deleted = click_log::Entity::delete_many()
                .filter(click_log::Column::Id.is_in(ids_to_delete))
                .exec(db)
                .await?
                .rows_affected;

            total_deleted += deleted;
            remaining = remaining.saturating_sub(deleted);

            debug!(
                "Deleted {} rows by row limit (total: {}, remaining: {})",
                deleted, total_deleted, remaining
            );
        }

        Ok(total_deleted)
    }

    /// 检查行数是否超过限制，并更新停止标志
    async fn check_and_update_stop_flag(&self) -> anyhow::Result<bool> {
        let db = self.storage.get_db();

        let current_count: i64 = click_log::Entity::find()
            .select_only()
            .column_as(click_log::Column::Id.count(), "count")
            .into_tuple()
            .one(db)
            .await?
            .unwrap_or(0);

        let should_stop = current_count as u64 >= self.max_log_rows;
        set_detailed_logging_stopped(should_stop);

        if should_stop {
            warn!(
                "Click logs count {} exceeds limit {}, stopping detailed logging",
                current_count, self.max_log_rows
            );
        } else {
            // 如果之前停止了，现在又可以继续了（比如手动清理了数据）
            debug!(
                "Click logs count {} is within limit {}",
                current_count, self.max_log_rows
            );
        }

        Ok(should_stop)
    }

    /// 清理过期的原始点击日志（分批删除避免长事务）
    async fn cleanup_raw_logs(&self) -> anyhow::Result<u64> {
        let db = self.storage.get_db();
        let cutoff = Utc::now() - Duration::days(self.raw_log_retention_days as i64);

        let mut total_deleted = 0u64;
        let mut iterations = 0;
        let max_iterations = 1000; // 防止无限循环

        loop {
            if iterations >= max_iterations {
                warn!(
                    "Raw log cleanup reached max iterations {} (deleted {} rows)",
                    max_iterations, total_deleted
                );
                break;
            }

            // 查找要删除的 ID 列表
            let ids_to_delete: Vec<i64> = click_log::Entity::find()
                .select_only()
                .column(click_log::Column::Id)
                .filter(click_log::Column::ClickedAt.lt(cutoff))
                .order_by_asc(click_log::Column::Id)
                .limit(self.batch_size)
                .into_tuple()
                .all(db)
                .await?;

            if ids_to_delete.is_empty() {
                break;
            }

            // 批量删除
            let deleted = click_log::Entity::delete_many()
                .filter(click_log::Column::Id.is_in(ids_to_delete.clone()))
                .exec(db)
                .await?
                .rows_affected;

            total_deleted += deleted;
            iterations += 1;

            debug!(
                "Raw log cleanup batch {}: deleted {} rows (total {})",
                iterations, deleted, total_deleted
            );

            // 如果删除的数量小于批量大小，说明已经没有更多数据
            if deleted < self.batch_size {
                break;
            }

            // 短暂暂停，避免对数据库造成过大压力
            tokio::time::sleep(StdDuration::from_millis(500)).await;
        }

        Ok(total_deleted)
    }

    /// 启动后台清理任务
    ///
    /// 每隔指定时间运行一次清理
    pub fn spawn_background_task(self: Arc<Self>, interval_hours: u64) {
        tokio::spawn(async move {
            let interval = StdDuration::from_secs(interval_hours * 60 * 60);

            // 首次运行延迟 5 分钟
            tokio::time::sleep(StdDuration::from_secs(300)).await;

            loop {
                if let Err(e) = self.run_cleanup().await {
                    error!("Data cleanup task failed: {}", e);
                }

                tokio::time::sleep(interval).await;
            }
        });

        info!(
            "Data cleanup background task started (interval: {} hours)",
            interval_hours
        );
    }
}
