use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 short_links 表
        manager
            .create_table(
                Table::create()
                    .table(ShortLink::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ShortLink::ShortCode)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ShortLink::TargetUrl)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ShortLink::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ShortLink::ExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ShortLink::Password)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ShortLink::ClickCount)
                            .big_integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建过期时间索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_expires_at")
                    .table(ShortLink::Table)
                    .col(ShortLink::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        // 创建创建时间索引
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_created_at")
                    .table(ShortLink::Table)
                    .col(ShortLink::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 删除索引
        manager
            .drop_index(Index::drop().name("idx_created_at").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_expires_at").to_owned())
            .await?;

        // 删除表
        manager
            .drop_table(Table::drop().table(ShortLink::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ShortLink {
    #[sea_orm(iden = "short_links")]
    Table,
    ShortCode,
    TargetUrl,
    CreatedAt,
    ExpiresAt,
    Password,
    ClickCount,
}
