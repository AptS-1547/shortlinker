//! 分析查询索引优化
//!
//! 为 click_logs 表添加索引以优化全局地理和来源查询

use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 索引：country + clicked_at（用于全局地理查询）
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_click_logs_country_time")
                    .table(ClickLogs::Table)
                    .col(ClickLogs::Country)
                    .col(ClickLogs::ClickedAt)
                    .to_owned(),
            )
            .await?;

        // 索引：referrer + clicked_at（用于全局来源查询）
        // MySQL 不支持在 TEXT 列上直接创建索引，需要使用前缀索引
        // sea-query 的 tuple 形式 .col((Column, prefix)) 在 PostgreSQL 上会生成无效语法，所以需要分后端处理
        let db = manager.get_database_backend();
        match db {
            sea_orm::DatabaseBackend::MySql => {
                manager
                    .create_index(
                        Index::create()
                            .if_not_exists()
                            .name("idx_click_logs_referrer_time")
                            .table(ClickLogs::Table)
                            .col((ClickLogs::Referrer, 191))
                            .col(ClickLogs::ClickedAt)
                            .to_owned(),
                    )
                    .await?;
            }
            _ => {
                // SQLite 和 PostgreSQL 支持在 TEXT 列上直接创建索引
                manager
                    .create_index(
                        Index::create()
                            .if_not_exists()
                            .name("idx_click_logs_referrer_time")
                            .table(ClickLogs::Table)
                            .col(ClickLogs::Referrer)
                            .col(ClickLogs::ClickedAt)
                            .to_owned(),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_click_logs_referrer_time")
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_click_logs_country_time")
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum ClickLogs {
    #[sea_orm(iden = "click_logs")]
    Table,
    Country,
    Referrer,
    ClickedAt,
}
