//! Source 和 Geo 字段索引优化
//!
//! 为 click_logs 表添加以下索引：
//! - idx_click_logs_source_time: source + clicked_at（用于来源统计查询）
//! - idx_click_logs_geo: country + city + clicked_at（用于地理分布查询）

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_database_backend();

        // 索引：source + clicked_at（用于来源统计查询）
        match db {
            sea_orm::DatabaseBackend::MySql => {
                // MySQL: source 是 VARCHAR(255)，可直接索引
                manager
                    .create_index(
                        Index::create()
                            .if_not_exists()
                            .name("idx_click_logs_source_time")
                            .table(ClickLogs::Table)
                            .col(ClickLogs::Source)
                            .col(ClickLogs::ClickedAt)
                            .to_owned(),
                    )
                    .await?;
            }
            _ => {
                // SQLite 和 PostgreSQL 直接创建索引
                manager
                    .create_index(
                        Index::create()
                            .if_not_exists()
                            .name("idx_click_logs_source_time")
                            .table(ClickLogs::Table)
                            .col(ClickLogs::Source)
                            .col(ClickLogs::ClickedAt)
                            .to_owned(),
                    )
                    .await?;
            }
        }

        // 索引：country + city + clicked_at（用于地理分布查询）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_click_logs_geo")
                    .table(ClickLogs::Table)
                    .col(ClickLogs::Country)
                    .col(ClickLogs::City)
                    .col(ClickLogs::ClickedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(Index::drop().name("idx_click_logs_geo").to_owned())
            .await?;

        manager
            .drop_index(Index::drop().name("idx_click_logs_source_time").to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ClickLogs {
    #[sea_orm(iden = "click_logs")]
    Table,
    Source,
    Country,
    City,
    ClickedAt,
}
