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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use sea_orm::ActiveValue;

    fn create_test_model() -> short_link::Model {
        short_link::Model {
            short_code: "abc123".to_string(),
            target_url: "https://example.com".to_string(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::days(7)),
            password: Some("hashed_password".to_string()),
            click_count: 42,
        }
    }

    fn create_test_shortlink() -> ShortLink {
        ShortLink {
            code: "xyz789".to_string(),
            target: "https://target.com".to_string(),
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::hours(24)),
            password: Some("secret".to_string()),
            click: 100,
        }
    }

    #[test]
    fn test_model_to_shortlink_basic() {
        let model = create_test_model();
        let expected_code = model.short_code.clone();
        let expected_target = model.target_url.clone();
        let expected_click = model.click_count as usize;

        let link = model_to_shortlink(model);

        assert_eq!(link.code, expected_code);
        assert_eq!(link.target, expected_target);
        assert_eq!(link.click, expected_click);
    }

    #[test]
    fn test_model_to_shortlink_with_none_fields() {
        let model = short_link::Model {
            short_code: "test".to_string(),
            target_url: "https://example.com".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            password: None,
            click_count: 0,
        };

        let link = model_to_shortlink(model);

        assert!(link.expires_at.is_none());
        assert!(link.password.is_none());
        assert_eq!(link.click, 0);
    }

    #[test]
    fn test_model_to_shortlink_negative_click_count() {
        let model = short_link::Model {
            short_code: "test".to_string(),
            target_url: "https://example.com".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            password: None,
            click_count: -10, // 负数应该被转换为 0
        };

        let link = model_to_shortlink(model);
        assert_eq!(link.click, 0);
    }

    #[test]
    fn test_shortlink_to_active_model_new() {
        let link = create_test_shortlink();
        let active_model = shortlink_to_active_model(&link, true);

        // 新建时，所有字段都应该被设置
        assert!(matches!(active_model.short_code, ActiveValue::Set(_)));
        assert!(matches!(active_model.target_url, ActiveValue::Set(_)));
        assert!(matches!(active_model.created_at, ActiveValue::Set(_)));
        assert!(matches!(active_model.expires_at, ActiveValue::Set(_)));
        assert!(matches!(active_model.password, ActiveValue::Set(_)));
        assert!(matches!(active_model.click_count, ActiveValue::Set(_)));

        // 验证值
        if let ActiveValue::Set(code) = active_model.short_code {
            assert_eq!(code, link.code);
        }
        if let ActiveValue::Set(target) = active_model.target_url {
            assert_eq!(target, link.target);
        }
        if let ActiveValue::Set(click) = active_model.click_count {
            assert_eq!(click, link.click as i64);
        }
    }

    #[test]
    fn test_shortlink_to_active_model_update() {
        let link = create_test_shortlink();
        let active_model = shortlink_to_active_model(&link, false);

        // 更新时，created_at 和 click_count 应该是 NotSet
        assert!(matches!(active_model.short_code, ActiveValue::Set(_)));
        assert!(matches!(active_model.target_url, ActiveValue::Set(_)));
        assert!(matches!(active_model.created_at, ActiveValue::NotSet));
        assert!(matches!(active_model.expires_at, ActiveValue::Set(_)));
        assert!(matches!(active_model.password, ActiveValue::Set(_)));
        assert!(matches!(active_model.click_count, ActiveValue::NotSet));
    }

    #[test]
    fn test_shortlink_to_active_model_with_none_fields() {
        let link = ShortLink {
            code: "test".to_string(),
            target: "https://example.com".to_string(),
            created_at: Utc::now(),
            expires_at: None,
            password: None,
            click: 0,
        };

        let active_model = shortlink_to_active_model(&link, true);

        if let ActiveValue::Set(expires) = active_model.expires_at {
            assert!(expires.is_none());
        }
        if let ActiveValue::Set(pwd) = active_model.password {
            assert!(pwd.is_none());
        }
    }

    #[test]
    fn test_roundtrip_conversion() {
        let original_model = create_test_model();
        let expected_code = original_model.short_code.clone();
        let expected_target = original_model.target_url.clone();

        // Model -> ShortLink
        let link = model_to_shortlink(original_model);

        // ShortLink -> ActiveModel (new)
        let active_model = shortlink_to_active_model(&link, true);

        // 验证关键字段保持一致
        if let ActiveValue::Set(code) = active_model.short_code {
            assert_eq!(code, expected_code);
        }
        if let ActiveValue::Set(target) = active_model.target_url {
            assert_eq!(target, expected_target);
        }
    }
}
