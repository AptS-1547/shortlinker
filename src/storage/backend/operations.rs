use sea_orm::DatabaseConnection;
use tracing::info;

use crate::errors::{Result, ShortlinkerError};
use crate::storage::ShortLink;
use migration::entities::short_link;

use super::converters::shortlink_to_active_model;

/// 使用 ON CONFLICT 的原子 upsert
pub async fn upsert(db: &DatabaseConnection, link: &ShortLink) -> Result<()> {
    use sea_orm::InsertResult;
    use sea_orm::{EntityTrait, sea_query::OnConflict};

    let active_model = shortlink_to_active_model(link, true);

    let result: std::result::Result<InsertResult<short_link::ActiveModel>, sea_orm::DbErr> =
        short_link::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(short_link::Column::ShortCode)
                    .update_columns([
                        short_link::Column::TargetUrl,
                        short_link::Column::ExpiresAt,
                        short_link::Column::Password,
                        short_link::Column::ClickCount,
                        short_link::Column::CreatedAt,
                    ])
                    .to_owned(),
            )
            .exec(db)
            .await;

    match result {
        Ok(_) => {
            info!("Short link upserted: {}", link.code);
            Ok(())
        }
        Err(e) => Err(ShortlinkerError::database_operation(format!(
            "Upsert 短链接 '{}' 失败 (target: {}): {}",
            link.code,
            if link.target.len() > 50 {
                format!("{}...", &link.target[..50])
            } else {
                link.target.clone()
            },
            e
        ))),
    }
}
