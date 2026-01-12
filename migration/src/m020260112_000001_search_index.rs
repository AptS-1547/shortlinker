use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::DatabaseBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();

        match backend {
            DatabaseBackend::Postgres => {
                // PostgreSQL: 使用 pg_trgm 扩展 + GIN 索引支持 LIKE '%keyword%' 搜索
                let conn = manager.get_connection();

                // 创建 pg_trgm 扩展（如果不存在）
                conn.execute_unprepared("CREATE EXTENSION IF NOT EXISTS pg_trgm")
                    .await?;

                // 为 short_code 创建 GIN 索引
                conn.execute_unprepared(
                    "CREATE INDEX IF NOT EXISTS idx_short_code_trgm ON short_links USING GIN (short_code gin_trgm_ops)",
                )
                .await?;

                // 为 target_url 创建 GIN 索引
                conn.execute_unprepared(
                    "CREATE INDEX IF NOT EXISTS idx_target_url_trgm ON short_links USING GIN (target_url gin_trgm_ops)",
                )
                .await?;
            }
            DatabaseBackend::MySql => {
                // MySQL: 使用 FULLTEXT 索引
                let conn = manager.get_connection();

                // 创建复合 FULLTEXT 索引
                // 注意：MySQL 的 FULLTEXT 索引不支持 IF NOT EXISTS，需要先检查
                conn.execute_unprepared(
                    "ALTER TABLE short_links ADD FULLTEXT INDEX idx_search_fulltext (short_code, target_url)",
                )
                .await
                .ok(); // 忽略错误（索引可能已存在）
            }
            DatabaseBackend::Sqlite => {
                // SQLite: 不做任何操作
                // SQLite 的 B-Tree 索引对 LIKE '%keyword%' 无效
                // FTS5 需要虚拟表，复杂度高，SQLite 通常用于小规模部署
            }
            _ => {
                // 其他数据库：不做任何操作
            }
        }

        // 所有数据库：添加复合索引 (expires_at, created_at DESC)
        // 用于优化 "获取未过期链接，按创建时间倒序" 查询
        let conn = manager.get_connection();
        conn.execute_unprepared(
            "CREATE INDEX IF NOT EXISTS idx_expires_created ON short_links (expires_at, created_at DESC)",
        )
        .await
        .ok(); // MySQL 8.0 以下不支持 DESC，忽略错误

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();
        let conn = manager.get_connection();

        // 先删除复合索引
        conn.execute_unprepared("DROP INDEX IF EXISTS idx_expires_created")
            .await
            .ok();

        match backend {
            DatabaseBackend::Postgres => {
                let conn = manager.get_connection();

                conn.execute_unprepared("DROP INDEX IF EXISTS idx_target_url_trgm")
                    .await?;

                conn.execute_unprepared("DROP INDEX IF EXISTS idx_short_code_trgm")
                    .await?;

                // 不删除 pg_trgm 扩展，可能被其他表使用
            }
            DatabaseBackend::MySql => {
                let conn = manager.get_connection();

                conn.execute_unprepared(
                    "ALTER TABLE short_links DROP INDEX idx_search_fulltext",
                )
                .await
                .ok(); // 忽略错误（索引可能不存在）
            }
            DatabaseBackend::Sqlite => {
                // 无操作
            }
            _ => {
                // 其他数据库：无操作
            }
        }

        Ok(())
    }
}
