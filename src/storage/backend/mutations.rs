//! Mutation operations for SeaOrmStorage
//!
//! This module contains all write database operations.

use std::collections::HashSet;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, sea_query::OnConflict};
use tracing::info;

use super::SeaOrmStorage;
use super::converters::shortlink_to_active_model;
use super::operations::upsert;
use crate::errors::{Result, ShortlinkerError};
use crate::storage::ShortLink;

use migration::entities::short_link;

impl SeaOrmStorage {
    pub async fn set(&self, link: ShortLink) -> Result<()> {
        let db = &self.db;

        aster_forge_db::retry::with_sea_orm_retry(
            &format!("set({})", link.code),
            self.retry_config,
            || async {
                // upsert() 返回原始 DbErr，retry 能正确识别连接错误
                upsert(db, &link).await
            },
        )
        .await
        .map_err(|e| {
            ShortlinkerError::database_operation(format!("Failed to set short link: {}", e))
        })?;

        self.invalidate_count_cache();
        Ok(())
    }

    pub async fn remove(&self, code: &str) -> Result<()> {
        let code_owned = code.to_string();

        let result = aster_forge_db::transaction::with_transaction_retry(
            &self.db,
            &self.retry_config,
            |txn| {
                let code = code_owned.clone();
                Box::pin(async move {
                    short_link::Entity::delete_by_id(code)
                        .exec(txn)
                        .await
                        .map_err(aster_forge_db::DbError::from)
                })
            },
            aster_forge_db::DbError::is_retryable,
        )
        .await
        .map_err(ShortlinkerError::from)?;

        if result.rows_affected == 0 {
            return Err(ShortlinkerError::not_found(format!(
                "Short link not found: {}",
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

        let requested = codes.to_vec();
        let (existing, not_found) = aster_forge_db::transaction::with_transaction_retry(
            &self.db,
            &self.retry_config,
            |txn| {
                let requested = requested.clone();
                Box::pin(async move {
                    let existing: Vec<String> = short_link::Entity::find()
                        .filter(short_link::Column::ShortCode.is_in(requested.iter().cloned()))
                        .all(txn)
                        .await
                        .map_err(aster_forge_db::DbError::from)?
                        .into_iter()
                        .map(|model| model.short_code)
                        .collect();
                    let existing_set: HashSet<&String> = existing.iter().collect();
                    let not_found = requested
                        .into_iter()
                        .filter(|code| !existing_set.contains(code))
                        .collect();

                    if !existing.is_empty() {
                        short_link::Entity::delete_many()
                            .filter(short_link::Column::ShortCode.is_in(existing.iter().cloned()))
                            .exec(txn)
                            .await
                            .map_err(aster_forge_db::DbError::from)?;
                    }
                    Ok((existing, not_found))
                })
            },
            aster_forge_db::DbError::is_retryable,
        )
        .await
        .map_err(ShortlinkerError::from)?;

        self.invalidate_count_cache();
        info!("Batch deleted {} links", existing.len());

        Ok((existing, not_found))
    }

    /// 批量设置链接（使用事务）
    pub async fn batch_set(&self, links: Vec<ShortLink>) -> Result<()> {
        if links.is_empty() {
            return Ok(());
        }

        aster_forge_db::transaction::with_transaction_retry(
            &self.db,
            &self.retry_config,
            |txn| {
                let links = links.clone();
                Box::pin(async move {
                    let active_models = links
                        .iter()
                        .map(|link| shortlink_to_active_model(link, true))
                        .collect::<Vec<_>>();
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
                        .exec(txn)
                        .await
                        .map_err(aster_forge_db::DbError::from)?;
                    Ok(())
                })
            },
            aster_forge_db::DbError::is_retryable,
        )
        .await
        .map_err(ShortlinkerError::from)?;

        self.invalidate_count_cache();
        info!("Batch inserted {} links", links.len());
        Ok(())
    }
}
