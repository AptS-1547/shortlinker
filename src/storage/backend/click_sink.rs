//! ClickSink implementation for SeaOrmStorage
//!
//! This module implements the click tracking flush functionality.
//!
//! # Security Note
//!
//! This module uses parameterized queries via `DatabaseBackend::build()` for SQL safety.
//! All `short_code` values are additionally validated via `utils::is_valid_short_code()`
//! as defense-in-depth against SQL injection.

use async_trait::async_trait;
use sea_orm::sea_query::{CaseStatement, Expr, Query};
use sea_orm::{ConnectionTrait, ExprTrait};
use tracing::debug;

use super::SeaOrmStorage;
use super::retry;
use crate::analytics::ClickSink;
use crate::utils::is_valid_short_code;

use migration::entities::short_link;

#[async_trait]
impl ClickSink for SeaOrmStorage {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
        if updates.is_empty() {
            return Ok(());
        }

        // 安全校验：确保所有 short_code 格式合法，防止 SQL 注入
        for (code, _) in &updates {
            if !is_valid_short_code(code) {
                return Err(anyhow::anyhow!(
                    "Invalid short_code format detected: '{}' - refusing to execute SQL",
                    code
                ));
            }
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

        // 使用参数化查询执行（SeaORM 内部自动 build 为带绑定参数的 Statement）
        let db = &self.db;
        let stmt_ref = &stmt;
        retry::with_retry("flush_clicks", self.retry_config, || async {
            db.execute(stmt_ref).await
        })
        .await
        .map_err(|e| anyhow::anyhow!("批量更新点击数失败（重试后仍失败）: {}", e))?;

        debug!(
            "点击计数已刷新到 {} 数据库 ({} 条记录)",
            self.backend_name.to_uppercase(),
            total_count
        );
        Ok(())
    }
}
