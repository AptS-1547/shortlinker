//! 删除 click_logs.user_agent 列
//!
//! 该字段已被 user_agent_hash 替代，不再需要存储原始 UA 字符串。

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ClickLogs::Table)
                    .drop_column(ClickLogs::UserAgent)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(ClickLogs::Table)
                    .add_column(ColumnDef::new(ClickLogs::UserAgent).text().null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum ClickLogs {
    #[sea_orm(iden = "click_logs")]
    Table,
    UserAgent,
}
