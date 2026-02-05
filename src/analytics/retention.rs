//! 数据清理任务
//!
//! 负责清理过期的点击日志和汇总数据，防止数据库无限增长。

use std::sync::Arc;
use std::time::Duration as StdDuration;

use chrono::{Duration, Utc};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use tracing::{debug, error, info, warn};

use crate::config::runtime_config::get_runtime_config;
use crate::storage::backend::SeaOrmStorage;
use migration::entities::click_log;

use super::RollupManager;

/// 清理报告
#[derive(Debug, Default)]
pub struct CleanupReport {
    /// 删除的原始点击日志数量
    pub raw_logs_deleted: u64,
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
}

impl DataRetentionTask {
    pub fn new(storage: Arc<SeaOrmStorage>, rollup_manager: Arc<RollupManager>) -> Self {
        // 从运行时配置读取保留天数
        let runtime_config = get_runtime_config();

        let raw_log_retention_days = runtime_config.get_u64_or("analytics.log_retention_days", 30);

        let hourly_retention_days = runtime_config.get_u64_or("analytics.hourly_retention_days", 7);

        let daily_retention_days = runtime_config.get_u64_or("analytics.daily_retention_days", 365);

        Self {
            storage,
            rollup_manager,
            raw_log_retention_days,
            hourly_retention_days,
            daily_retention_days,
            batch_size: 10000,
        }
    }

    /// 运行完整的清理流程
    pub async fn run_cleanup(&self) -> anyhow::Result<CleanupReport> {
        let mut report = CleanupReport::default();

        // 1. 清理原始点击日志（分批删除）
        match self.cleanup_raw_logs().await {
            Ok(deleted) => {
                report.raw_logs_deleted = deleted;
            }
            Err(e) => {
                error!("Failed to clean up raw click logs: {}", e);
            }
        }

        // 2. 清理汇总数据
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
            "Data cleanup completed: raw logs {}, hourly rollups {}, daily rollups {}",
            report.raw_logs_deleted, report.hourly_stats_deleted, report.daily_stats_deleted
        );

        Ok(report)
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
            tokio::time::sleep(StdDuration::from_millis(100)).await;
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
