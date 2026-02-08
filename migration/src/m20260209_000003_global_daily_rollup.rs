//! 全局天级点击统计汇总表迁移
//!
//! 新增 `click_stats_global_daily` 表，用于存储全局天级汇总数据，
//! 优化 Day/Week/Month 粒度的趋势查询性能。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ClickStatsGlobalDaily::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClickStatsGlobalDaily::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalDaily::DayBucket)
                            .date()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalDaily::TotalClicks)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalDaily::UniqueLinks)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalDaily::TopReferrers)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalDaily::TopCountries)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalDaily::TopSources)
                            .text()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 唯一索引：day_bucket（每天只有一条全局记录）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_stats_global_daily_bucket")
                    .table(ClickStatsGlobalDaily::Table)
                    .col(ClickStatsGlobalDaily::DayBucket)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_stats_global_daily_bucket")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ClickStatsGlobalDaily::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum ClickStatsGlobalDaily {
    #[sea_orm(iden = "click_stats_global_daily")]
    Table,
    Id,
    DayBucket,
    TotalClicks,
    UniqueLinks,
    TopReferrers,
    TopCountries,
    TopSources,
}
