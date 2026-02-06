//! UTM Source 追踪字段迁移
//!
//! 添加 source 字段用于区分 QR Code / 直接访问 / Referrer 来源：
//! - click_logs: 添加 source 列
//! - click_stats_hourly: 添加 source_counts 列
//! - click_stats_daily: 添加 top_sources 列

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. click_logs 添加 source 列
        manager
            .alter_table(
                Table::alter()
                    .table(ClickLogs::Table)
                    .add_column(ColumnDef::new(ClickLogs::Source).string_len(255).null())
                    .to_owned(),
            )
            .await?;

        // 2. click_stats_hourly 添加 source_counts 列
        manager
            .alter_table(
                Table::alter()
                    .table(ClickStatsHourly::Table)
                    .add_column(ColumnDef::new(ClickStatsHourly::SourceCounts).text().null())
                    .to_owned(),
            )
            .await?;

        // 3. click_stats_daily 添加 top_sources 列
        manager
            .alter_table(
                Table::alter()
                    .table(ClickStatsDaily::Table)
                    .add_column(ColumnDef::new(ClickStatsDaily::TopSources).text().null())
                    .to_owned(),
            )
            .await?;

        // 4. click_stats_daily 添加 unique_sources 列
        manager
            .alter_table(
                Table::alter()
                    .table(ClickStatsDaily::Table)
                    .add_column(
                        ColumnDef::new(ClickStatsDaily::UniqueSources)
                            .integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除 click_stats_daily.unique_sources
        manager
            .alter_table(
                Table::alter()
                    .table(ClickStatsDaily::Table)
                    .drop_column(ClickStatsDaily::UniqueSources)
                    .to_owned(),
            )
            .await?;

        // 删除 click_stats_daily.top_sources
        manager
            .alter_table(
                Table::alter()
                    .table(ClickStatsDaily::Table)
                    .drop_column(ClickStatsDaily::TopSources)
                    .to_owned(),
            )
            .await?;

        // 删除 click_stats_hourly.source_counts
        manager
            .alter_table(
                Table::alter()
                    .table(ClickStatsHourly::Table)
                    .drop_column(ClickStatsHourly::SourceCounts)
                    .to_owned(),
            )
            .await?;

        // 删除 click_logs.source
        manager
            .alter_table(
                Table::alter()
                    .table(ClickLogs::Table)
                    .drop_column(ClickLogs::Source)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ClickLogs {
    #[sea_orm(iden = "click_logs")]
    Table,
    Source,
}

#[derive(DeriveIden)]
enum ClickStatsHourly {
    #[sea_orm(iden = "click_stats_hourly")]
    Table,
    SourceCounts,
}

#[derive(DeriveIden)]
enum ClickStatsDaily {
    #[sea_orm(iden = "click_stats_daily")]
    Table,
    TopSources,
    UniqueSources,
}
