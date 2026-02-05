//! 点击统计汇总表迁移
//!
//! 创建预聚合的汇总表，提升分析查询性能：
//! - click_stats_hourly: 小时级链接点击汇总
//! - click_stats_daily: 天级链接点击汇总
//! - click_stats_global_hourly: 全局小时汇总

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. 创建 click_stats_hourly 表
        manager
            .create_table(
                Table::create()
                    .table(ClickStatsHourly::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClickStatsHourly::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsHourly::ShortCode)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsHourly::HourBucket)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsHourly::ClickCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ClickStatsHourly::ReferrerCounts)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsHourly::CountryCounts)
                            .text()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 唯一索引：short_code + hour_bucket
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_stats_hourly_code_bucket")
                    .table(ClickStatsHourly::Table)
                    .col(ClickStatsHourly::ShortCode)
                    .col(ClickStatsHourly::HourBucket)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 索引：hour_bucket（用于清理和范围查询）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_stats_hourly_bucket")
                    .table(ClickStatsHourly::Table)
                    .col(ClickStatsHourly::HourBucket)
                    .to_owned(),
            )
            .await?;

        // 2. 创建 click_stats_daily 表
        manager
            .create_table(
                Table::create()
                    .table(ClickStatsDaily::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClickStatsDaily::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsDaily::ShortCode)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(ColumnDef::new(ClickStatsDaily::DayBucket).date().not_null())
                    .col(
                        ColumnDef::new(ClickStatsDaily::ClickCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ClickStatsDaily::UniqueReferrers)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsDaily::UniqueCountries)
                            .integer()
                            .null(),
                    )
                    .col(ColumnDef::new(ClickStatsDaily::TopReferrers).text().null())
                    .col(ColumnDef::new(ClickStatsDaily::TopCountries).text().null())
                    .to_owned(),
            )
            .await?;

        // 唯一索引：short_code + day_bucket
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_stats_daily_code_bucket")
                    .table(ClickStatsDaily::Table)
                    .col(ClickStatsDaily::ShortCode)
                    .col(ClickStatsDaily::DayBucket)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // 索引：day_bucket
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_stats_daily_bucket")
                    .table(ClickStatsDaily::Table)
                    .col(ClickStatsDaily::DayBucket)
                    .to_owned(),
            )
            .await?;

        // 3. 创建 click_stats_global_hourly 表
        manager
            .create_table(
                Table::create()
                    .table(ClickStatsGlobalHourly::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClickStatsGlobalHourly::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalHourly::HourBucket)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalHourly::TotalClicks)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalHourly::UniqueLinks)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalHourly::TopReferrers)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ClickStatsGlobalHourly::TopCountries)
                            .text()
                            .null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 唯一索引：hour_bucket
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_stats_global_hourly_bucket")
                    .table(ClickStatsGlobalHourly::Table)
                    .col(ClickStatsGlobalHourly::HourBucket)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除 click_stats_global_hourly
        manager
            .drop_index(
                Index::drop()
                    .name("idx_stats_global_hourly_bucket")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(ClickStatsGlobalHourly::Table)
                    .to_owned(),
            )
            .await?;

        // 删除 click_stats_daily
        manager
            .drop_index(Index::drop().name("idx_stats_daily_bucket").to_owned())
            .await?;
        manager
            .drop_index(Index::drop().name("idx_stats_daily_code_bucket").to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(ClickStatsDaily::Table).to_owned())
            .await?;

        // 删除 click_stats_hourly
        manager
            .drop_index(Index::drop().name("idx_stats_hourly_bucket").to_owned())
            .await?;
        manager
            .drop_index(
                Index::drop()
                    .name("idx_stats_hourly_code_bucket")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(ClickStatsHourly::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ClickStatsHourly {
    #[sea_orm(iden = "click_stats_hourly")]
    Table,
    Id,
    ShortCode,
    HourBucket,
    ClickCount,
    ReferrerCounts,
    CountryCounts,
}

#[derive(DeriveIden)]
enum ClickStatsDaily {
    #[sea_orm(iden = "click_stats_daily")]
    Table,
    Id,
    ShortCode,
    DayBucket,
    ClickCount,
    UniqueReferrers,
    UniqueCountries,
    TopReferrers,
    TopCountries,
}

#[derive(DeriveIden)]
enum ClickStatsGlobalHourly {
    #[sea_orm(iden = "click_stats_global_hourly")]
    Table,
    Id,
    HourBucket,
    TotalClicks,
    UniqueLinks,
    TopReferrers,
    TopCountries,
}
