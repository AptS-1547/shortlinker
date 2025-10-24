use sea_orm::DatabaseConnection;
use tracing::{info, warn};

use crate::errors::{Result, ShortlinkerError};
use crate::storage::ShortLink;
use migration::entities::short_link;

use super::converters::shortlink_to_active_model;

/// 使用 ON CONFLICT 的原子 upsert
pub async fn upsert_with_on_conflict(db: &DatabaseConnection, link: &ShortLink) -> Result<()> {
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
            "Upsert 短链接失败: {}",
            e
        ))),
    }
}

/// 使用 try-insert-then-update 的 upsert
pub async fn upsert_with_fallback(db: &DatabaseConnection, link: &ShortLink) -> Result<()> {
    use sea_orm::ActiveModelTrait;

    // 先尝试插入
    let active_model = shortlink_to_active_model(link, true);
    let insert_result = active_model.clone().insert(db).await;

    match insert_result {
        Ok(_) => {
            info!("Short link created: {}", link.code);
            Ok(())
        }
        Err(sea_orm::DbErr::Exec(sea_orm::RuntimeErr::SqlxError(sqlx_err))) => {
            // 检查是否是唯一约束冲突错误
            if is_unique_violation(&sqlx_err) {
                // 如果是唯一冲突，执行更新
                let update_model = shortlink_to_active_model(link, false);
                update_model.update(db).await.map_err(|e| {
                    ShortlinkerError::database_operation(format!("更新短链接失败: {}", e))
                })?;
                info!("Short link updated: {}", link.code);
                Ok(())
            } else {
                // 其他错误直接返回
                Err(ShortlinkerError::database_operation(format!(
                    "插入短链接失败: {}",
                    sqlx_err
                )))
            }
        }
        Err(e) => Err(ShortlinkerError::database_operation(format!(
            "插入短链接失败: {}",
            e
        ))),
    }
}

/// 判断是否是唯一约束冲突错误
fn is_unique_violation(err: &sea_orm::sqlx::Error) -> bool {
    use sea_orm::sqlx::Error;

    match err {
        Error::Database(db_err) => {
            let code = db_err.code();
            // SQLite: SQLITE_CONSTRAINT (code 2067)
            // MySQL: ER_DUP_ENTRY (code 1062)
            // PostgreSQL: unique_violation (code 23505)
            code.as_ref()
                .map(|c| {
                    c == "2067"  // SQLite
                        || c == "1062"  // MySQL
                        || c == "23505" // PostgreSQL
                })
                .unwrap_or(false)
        }
        _ => false,
    }
}

/// 根据 backend 选择 upsert 策略
pub async fn upsert(db: &DatabaseConnection, backend_name: &str, link: &ShortLink) -> Result<()> {
    match backend_name {
        "sqlite" | "postgres" | "mysql" => {
            // These backends support ON CONFLICT or equivalent upsert syntax.
            upsert_with_on_conflict(db, link).await
        }
        _ => {
            // Fallback for other databases, if any.
            warn!("Using fallback upsert for backend: {}", backend_name);
            upsert_with_fallback(db, link).await
        }
    }
}
