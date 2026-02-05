//! Mutation operations for SeaOrmStorage
//!
//! This module contains all write database operations.

use std::collections::HashSet;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionTrait, sea_query::OnConflict};
use tracing::info;

use super::SeaOrmStorage;
use super::converters::shortlink_to_active_model;
use super::operations::upsert;
use super::retry;
use crate::errors::{Result, ShortlinkerError};
use crate::storage::ShortLink;

use migration::entities::short_link;

impl SeaOrmStorage {
    pub async fn set(&self, link: ShortLink) -> Result<()> {
        let db = &self.db;

        retry::with_retry(
            &format!("set({})", link.code),
            self.retry_config,
            || async {
                upsert(db, &link).await.map_err(|e| {
                    // 转换为 DbErr 以便重试机制判断
                    sea_orm::DbErr::Custom(e.to_string())
                })
            },
        )
        .await
        .map_err(|e| ShortlinkerError::database_operation(format!("设置短链接失败: {}", e)))?;

        self.invalidate_count_cache();
        Ok(())
    }

    pub async fn remove(&self, code: &str) -> Result<()> {
        let db = &self.db;
        let code_owned = code.to_string();

        let result = retry::with_retry(&format!("remove({})", code), self.retry_config, || async {
            short_link::Entity::delete_by_id(&code_owned).exec(db).await
        })
        .await
        .map_err(|e| ShortlinkerError::database_operation(format!("删除短链接失败: {}", e)))?;

        if result.rows_affected == 0 {
            return Err(ShortlinkerError::not_found(format!(
                "短链接不存在: {}",
                code
            )));
        }

        self.invalidate_count_cache();
        info!("Short link deleted: {}", code);
        Ok(())
    }

    /// 批量删除链接
    /// 返回 (成功删除的 codes, 不存在的 codes)
    pub async fn batch_remove(&self, codes: &[String]) -> Result<(Vec<String>, Vec<String>)> {
        if codes.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        // 先查询哪些存在
        let existing: Vec<String> = short_link::Entity::find()
            .filter(short_link::Column::ShortCode.is_in(codes.iter().cloned()))
            .all(&self.db)
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("查询失败: {}", e)))?
            .into_iter()
            .map(|m| m.short_code)
            .collect();

        // 使用 HashSet 优化 contains 查找（O(1) vs O(n)）
        let existing_set: HashSet<&String> = existing.iter().collect();
        let not_found: Vec<String> = codes
            .iter()
            .filter(|c| !existing_set.contains(c))
            .cloned()
            .collect();

        if existing.is_empty() {
            return Ok((Vec::new(), not_found));
        }

        // 批量删除
        short_link::Entity::delete_many()
            .filter(short_link::Column::ShortCode.is_in(existing.iter().cloned()))
            .exec(&self.db)
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("批量删除失败: {}", e)))?;

        self.invalidate_count_cache();
        info!("Batch deleted {} links", existing.len());

        Ok((existing, not_found))
    }

    /// 批量设置链接（使用事务）
    pub async fn batch_set(&self, links: Vec<ShortLink>) -> Result<()> {
        if links.is_empty() {
            return Ok(());
        }

        let txn =
            self.db.begin().await.map_err(|e| {
                ShortlinkerError::database_operation(format!("开始事务失败: {}", e))
            })?;

        // 构建批量 ActiveModel
        let active_models: Vec<short_link::ActiveModel> = links
            .iter()
            .map(|link| shortlink_to_active_model(link, true))
            .collect();

        // 使用 insert_many with on_conflict
        short_link::Entity::insert_many(active_models)
            .on_conflict(
                OnConflict::column(short_link::Column::ShortCode)
                    .update_columns([
                        short_link::Column::TargetUrl,
                        short_link::Column::ExpiresAt,
                        short_link::Column::Password,
                        short_link::Column::CreatedAt,
                        short_link::Column::ClickCount,
                    ])
                    .to_owned(),
            )
            .exec(&txn)
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("批量插入失败: {}", e)))?;

        txn.commit()
            .await
            .map_err(|e| ShortlinkerError::database_operation(format!("提交事务失败: {}", e)))?;

        self.invalidate_count_cache();
        info!("Batch inserted {} links", links.len());
        Ok(())
    }
}
