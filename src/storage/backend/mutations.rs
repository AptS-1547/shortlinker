//! Mutation operations for SeaOrmStorage
//!
//! This module contains all write database operations.

use sea_orm::{EntityTrait, TransactionTrait, sea_query::OnConflict};
use tracing::info;

use super::SeaOrmStorage;
use super::converters::shortlink_to_active_model;
use super::operations::upsert;
use crate::errors::{Result, ShortlinkerError};
use crate::storage::ShortLink;

use migration::entities::short_link;

impl SeaOrmStorage {
    pub async fn set(&self, link: ShortLink) -> Result<()> {
        upsert(&self.db, &link).await?;
        self.invalidate_count_cache();
        Ok(())
    }

    pub async fn remove(&self, code: &str) -> Result<()> {
        let result = short_link::Entity::delete_by_id(code)
            .exec(&self.db)
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
        info!("批量插入 {} 条链接成功", links.len());
        Ok(())
    }
}
