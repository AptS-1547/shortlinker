use crate::storage::ShortLink;
use migration::entities::short_link;

/// 将 Sea-ORM Model 转换为 ShortLink
pub fn model_to_shortlink(model: short_link::Model) -> ShortLink {
    ShortLink {
        code: model.short_code,
        target: model.target_url,
        created_at: model.created_at,
        expires_at: model.expires_at,
        password: model.password,
        click: model.click_count.max(0) as usize,
    }
}

/// 将 ShortLink 转换为 ActiveModel（用于插入/更新）
pub fn shortlink_to_active_model(link: &ShortLink, is_new: bool) -> short_link::ActiveModel {
    use sea_orm::ActiveValue::*;

    short_link::ActiveModel {
        short_code: Set(link.code.clone()),
        target_url: Set(link.target.clone()),
        created_at: if is_new { Set(link.created_at) } else { NotSet },
        expires_at: Set(link.expires_at),
        password: Set(link.password.clone()),
        click_count: if is_new {
            Set(link.click as i64)
        } else {
            NotSet
        },
    }
}
