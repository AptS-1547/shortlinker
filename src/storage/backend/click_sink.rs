//! ClickSink implementation for SeaOrmStorage
//!
//! This module implements the click tracking flush functionality.

use async_trait::async_trait;
use sea_orm::sea_query::{
    CaseStatement, Expr, MysqlQueryBuilder, PostgresQueryBuilder, Query, SqliteQueryBuilder,
};
use sea_orm::{ConnectionTrait, DatabaseBackend, ExprTrait};
use tracing::debug;

use super::SeaOrmStorage;
use crate::analytics::ClickSink;

use migration::entities::short_link;

#[async_trait]
impl ClickSink for SeaOrmStorage {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
        if updates.is_empty() {
            return Ok(());
        }

        let total_count = updates.len();

        // 构建 CASE WHEN 表达式（跨平台兼容）
        let mut case_stmt = CaseStatement::new();
        let mut codes: Vec<String> = Vec::with_capacity(total_count);

        for (code, count) in &updates {
            case_stmt = case_stmt.case(
                Expr::col(short_link::Column::ShortCode).eq(Expr::val(code.as_str())),
                Expr::col(short_link::Column::ClickCount).add(Expr::val(*count as i64)),
            );
            codes.push(code.clone());
        }
        // 不匹配的保持原值
        case_stmt = case_stmt.finally(Expr::col(short_link::Column::ClickCount));

        // 构建 UPDATE 语句
        let stmt = Query::update()
            .table(short_link::Entity)
            .value(short_link::Column::ClickCount, case_stmt)
            .and_where(Expr::col(short_link::Column::ShortCode).is_in(codes))
            .to_owned();

        // 使用 to_string 生成内联值的 SQL（根据数据库类型选择对应的 QueryBuilder）
        let sql = match self.db.get_database_backend() {
            DatabaseBackend::Sqlite => stmt.to_string(SqliteQueryBuilder),
            DatabaseBackend::MySql => stmt.to_string(MysqlQueryBuilder),
            DatabaseBackend::Postgres => stmt.to_string(PostgresQueryBuilder),
            _ => stmt.to_string(SqliteQueryBuilder), // fallback to SQLite
        };

        self.db
            .execute_unprepared(&sql)
            .await
            .map_err(|e| anyhow::anyhow!("批量更新点击数失败: {}", e))?;

        debug!(
            "点击计数已刷新到 {} 数据库 ({} 条记录)",
            self.backend_name.to_uppercase(),
            total_count
        );
        Ok(())
    }
}
