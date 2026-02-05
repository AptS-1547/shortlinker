//! UserAgent 去重表迁移
//!
//! 创建 user_agents 查找表，用于存储 UA 字符串的 hash 映射，
//! 并在 click_logs 表添加 user_agent_hash 列引用。
//!
//! 注意：历史数据迁移在应用启动时由 UserAgentStore 自动处理。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. 创建 user_agents 查找表
        manager
            .create_table(
                Table::create()
                    .table(UserAgents::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserAgents::Hash)
                            .char_len(16) // xxHash64 hex 表示
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(UserAgents::UserAgentString)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserAgents::FirstSeen)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserAgents::LastSeen)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 2. 在 click_logs 表添加 user_agent_hash 列
        manager
            .alter_table(
                Table::alter()
                    .table(ClickLogs::Table)
                    .add_column(ColumnDef::new(ClickLogs::UserAgentHash).char_len(16).null())
                    .to_owned(),
            )
            .await?;

        // 3. 创建索引用于 JOIN 查询
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_click_logs_ua_hash")
                    .table(ClickLogs::Table)
                    .col(ClickLogs::UserAgentHash)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除索引
        manager
            .drop_index(Index::drop().name("idx_click_logs_ua_hash").to_owned())
            .await?;

        // 删除 click_logs.user_agent_hash 列
        manager
            .alter_table(
                Table::alter()
                    .table(ClickLogs::Table)
                    .drop_column(ClickLogs::UserAgentHash)
                    .to_owned(),
            )
            .await?;

        // 删除 user_agents 表
        manager
            .drop_table(Table::drop().table(UserAgents::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum UserAgents {
    #[sea_orm(iden = "user_agents")]
    Table,
    Hash,
    UserAgentString,
    FirstSeen,
    LastSeen,
}

#[derive(DeriveIden)]
enum ClickLogs {
    #[sea_orm(iden = "click_logs")]
    Table,
    UserAgentHash,
}
