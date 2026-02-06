//! 分析查询索引优化 (第二批)
//!
//! 添加以下索引：
//! - click_stats_hourly.short_code 单列索引（用于单链接查询优化）
//! - user_agents.is_bot 索引（用于 Bot 统计查询）

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 索引：click_stats_hourly.short_code（单链接分析查询优化）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_stats_hourly_code")
                    .table(ClickStatsHourly::Table)
                    .col(ClickStatsHourly::ShortCode)
                    .to_owned(),
            )
            .await?;

        // 索引：user_agents.is_bot（Bot 统计查询优化）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_user_agents_is_bot")
                    .table(UserAgents::Table)
                    .col(UserAgents::IsBot)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_user_agents_is_bot").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_stats_hourly_code").to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ClickStatsHourly {
    #[sea_orm(iden = "click_stats_hourly")]
    Table,
    ShortCode,
}

#[derive(DeriveIden)]
enum UserAgents {
    #[sea_orm(iden = "user_agents")]
    Table,
    IsBot,
}
