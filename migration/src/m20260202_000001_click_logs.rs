//! 点击日志表迁移
//!
//! 创建 click_logs 表用于存储详细的点击统计信息，包括：
//! - 时间戳
//! - 来源 (referrer)
//! - 用户代理 (user_agent)
//! - IP 地址
//! - 地理位置信息 (country, city)

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 click_logs 表
        manager
            .create_table(
                Table::create()
                    .table(ClickLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ClickLogs::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ClickLogs::ShortCode)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ClickLogs::ClickedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ClickLogs::Referrer).text().null())
                    .col(ColumnDef::new(ClickLogs::UserAgent).text().null())
                    .col(ColumnDef::new(ClickLogs::IpAddress).string_len(45).null())
                    .col(ColumnDef::new(ClickLogs::Country).string_len(2).null())
                    .col(ColumnDef::new(ClickLogs::City).string_len(100).null())
                    .to_owned(),
            )
            .await?;

        // 创建 short_code 索引（用于单链接查询）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_click_logs_short_code")
                    .table(ClickLogs::Table)
                    .col(ClickLogs::ShortCode)
                    .to_owned(),
            )
            .await?;

        // 创建 clicked_at 索引（用于时间范围查询）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_click_logs_clicked_at")
                    .table(ClickLogs::Table)
                    .col(ClickLogs::ClickedAt)
                    .to_owned(),
            )
            .await?;

        // 创建复合索引（用于单链接时间序列查询）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_click_logs_code_time")
                    .table(ClickLogs::Table)
                    .col(ClickLogs::ShortCode)
                    .col(ClickLogs::ClickedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除索引
        manager
            .drop_index(Index::drop().name("idx_click_logs_code_time").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_click_logs_clicked_at").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_click_logs_short_code").to_owned())
            .await?;

        // 删除表
        manager
            .drop_table(Table::drop().table(ClickLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ClickLogs {
    #[sea_orm(iden = "click_logs")]
    Table,
    Id,
    ShortCode,
    ClickedAt,
    Referrer,
    UserAgent,
    IpAddress,
    Country,
    City,
}
