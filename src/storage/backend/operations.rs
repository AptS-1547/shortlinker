use sea_orm::{DatabaseConnection, DbErr, EntityTrait, sea_query::OnConflict};
use tracing::info;

use crate::storage::ShortLink;
use migration::entities::short_link;

use super::converters::shortlink_to_active_model;

/// 使用 ON CONFLICT 的原子 upsert
///
/// 返回原始 `DbErr` 以便调用方的 retry 机制能识别连接错误类型
pub async fn upsert(db: &DatabaseConnection, link: &ShortLink) -> Result<(), DbErr> {
    let active_model = shortlink_to_active_model(link, true);

    short_link::Entity::insert(active_model)
        .on_conflict(
            OnConflict::column(short_link::Column::ShortCode)
                .update_columns([
                    short_link::Column::TargetUrl,
                    short_link::Column::ExpiresAt,
                    short_link::Column::Password,
                    short_link::Column::ClickCount,
                ])
                .to_owned(),
        )
        .exec(db)
        .await?;

    info!("Short link upserted: {}", link.code);
    Ok(())
}
