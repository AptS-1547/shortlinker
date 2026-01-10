use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 system_config 表
        manager
            .create_table(
                Table::create()
                    .table(SystemConfig::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemConfig::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SystemConfig::Value).text().not_null())
                    .col(
                        ColumnDef::new(SystemConfig::ValueType)
                            .string()
                            .not_null()
                            .default("string"),
                    )
                    .col(
                        ColumnDef::new(SystemConfig::RequiresRestart)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SystemConfig::IsSensitive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SystemConfig::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 updated_at 索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_config_updated")
                    .table(SystemConfig::Table)
                    .col(SystemConfig::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        // 创建 config_history 表
        manager
            .create_table(
                Table::create()
                    .table(ConfigHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ConfigHistory::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ConfigHistory::ConfigKey).string().not_null())
                    .col(ColumnDef::new(ConfigHistory::OldValue).text().null())
                    .col(ColumnDef::new(ConfigHistory::NewValue).text().not_null())
                    .col(
                        ColumnDef::new(ConfigHistory::ChangedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ConfigHistory::ChangedBy).string().null())
                    .to_owned(),
            )
            .await?;

        // 创建 config_key 索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_history_key")
                    .table(ConfigHistory::Table)
                    .col(ConfigHistory::ConfigKey)
                    .to_owned(),
            )
            .await?;

        // 创建 changed_at 索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_history_time")
                    .table(ConfigHistory::Table)
                    .col(ConfigHistory::ChangedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除 config_history 索引
        manager
            .drop_index(Index::drop().name("idx_history_time").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_history_key").to_owned())
            .await?;

        // 删除 config_history 表
        manager
            .drop_table(Table::drop().table(ConfigHistory::Table).to_owned())
            .await?;

        // 删除 system_config 索引
        manager
            .drop_index(Index::drop().name("idx_config_updated").to_owned())
            .await?;

        // 删除 system_config 表
        manager
            .drop_table(Table::drop().table(SystemConfig::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum SystemConfig {
    #[sea_orm(iden = "system_config")]
    Table,
    Key,
    Value,
    ValueType,
    RequiresRestart,
    IsSensitive,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum ConfigHistory {
    #[sea_orm(iden = "config_history")]
    Table,
    Id,
    ConfigKey,
    OldValue,
    NewValue,
    ChangedAt,
    ChangedBy,
}
