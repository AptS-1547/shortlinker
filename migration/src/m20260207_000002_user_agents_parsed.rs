//! UserAgent 解析字段迁移
//!
//! 扩展 user_agents 表，添加解析后的字段（浏览器、OS、设备类型等）

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 添加解析字段
        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .add_column(ColumnDef::new(UserAgents::BrowserName).string_len(50).null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .add_column(ColumnDef::new(UserAgents::BrowserVersion).string_len(20).null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .add_column(ColumnDef::new(UserAgents::OsName).string_len(50).null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .add_column(ColumnDef::new(UserAgents::OsVersion).string_len(20).null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .add_column(ColumnDef::new(UserAgents::DeviceCategory).string_len(20).null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .add_column(ColumnDef::new(UserAgents::DeviceVendor).string_len(50).null())
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .add_column(
                        ColumnDef::new(UserAgents::IsBot)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建索引用于统计查询
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_user_agents_device_category")
                    .table(UserAgents::Table)
                    .col(UserAgents::DeviceCategory)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_user_agents_browser_name")
                    .table(UserAgents::Table)
                    .col(UserAgents::BrowserName)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_user_agents_os_name")
                    .table(UserAgents::Table)
                    .col(UserAgents::OsName)
                    .to_owned(),
            )
            .await?;

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
        // 删除索引
        manager
            .drop_index(Index::drop().name("idx_user_agents_is_bot").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_user_agents_os_name").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_user_agents_browser_name").to_owned())
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_agents_device_category")
                    .to_owned(),
            )
            .await?;

        // 删除字段
        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .drop_column(UserAgents::IsBot)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .drop_column(UserAgents::DeviceVendor)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .drop_column(UserAgents::DeviceCategory)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .drop_column(UserAgents::OsVersion)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .drop_column(UserAgents::OsName)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .drop_column(UserAgents::BrowserVersion)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(UserAgents::Table)
                    .drop_column(UserAgents::BrowserName)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserAgents {
    #[sea_orm(iden = "user_agents")]
    Table,
    BrowserName,
    BrowserVersion,
    OsName,
    OsVersion,
    DeviceCategory,
    DeviceVendor,
    IsBot,
}
