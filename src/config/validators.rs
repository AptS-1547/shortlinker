//! 配置值验证模块
//!
//! 基于 schema 验证配置值是否合法。

use super::schema::get_schema;

/// 根据配置 key 验证值是否合法
///
/// 基于 schema 进行验证：
/// - 如果配置有 `enum_options`，验证值是否在选项列表中
/// - 如果配置有 `array_item_options`，验证 JSON 数组中的每个元素是否合法
/// - 没有 schema 的配置不做验证
pub fn validate_config_value(key: &str, value: &str) -> Result<(), String> {
    let schema = match get_schema(key) {
        Some(s) => s,
        None => return Ok(()), // 没有 schema 的配置不验证
    };

    // 验证 enum 类型
    if let Some(ref options) = schema.enum_options {
        let valid_values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
        if !valid_values
            .iter()
            .any(|v| v.eq_ignore_ascii_case(value))
        {
            return Err(format!(
                "Invalid value '{}'. Valid options: {:?}",
                value, valid_values
            ));
        }
    }

    // 验证 JSON 数组中的 enum 元素
    if let Some(ref item_options) = schema.array_item_options {
        let items: Vec<String> = serde_json::from_str(value).map_err(|e| {
            format!(
                "Invalid JSON format: {}. Expected array like [\"GET\", \"POST\", ...]",
                e
            )
        })?;

        let valid_values: Vec<&str> = item_options.iter().map(|o| o.value.as_str()).collect();
        let mut invalid_items = Vec::new();

        for item in &items {
            if !valid_values
                .iter()
                .any(|v| v.eq_ignore_ascii_case(item))
            {
                invalid_items.push(item.clone());
            }
        }

        if !invalid_items.is_empty() {
            return Err(format!(
                "Invalid items: {:?}. Valid options: {:?}",
                invalid_items, valid_values
            ));
        }
    }

    Ok(())
}

/// 获取 enum 配置的默认值（用于迁移时自动修正不合法的值）
pub fn get_enum_default_value(key: &str) -> Option<String> {
    get_schema(key).map(|s| s.default_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::runtime_config::keys;

    #[test]
    fn test_validate_same_site_policy() {
        // 合法值
        assert!(validate_config_value(keys::API_COOKIE_SAME_SITE, "Strict").is_ok());
        assert!(validate_config_value(keys::API_COOKIE_SAME_SITE, "Lax").is_ok());
        assert!(validate_config_value(keys::API_COOKIE_SAME_SITE, "None").is_ok());
        // 大小写不敏感
        assert!(validate_config_value(keys::API_COOKIE_SAME_SITE, "strict").is_ok());
        assert!(validate_config_value(keys::API_COOKIE_SAME_SITE, "LAX").is_ok());

        // 非法值
        assert!(validate_config_value(keys::API_COOKIE_SAME_SITE, "invalid").is_err());
        assert!(validate_config_value(keys::API_COOKIE_SAME_SITE, "").is_err());
    }

    #[test]
    fn test_validate_http_methods() {
        // 合法值
        assert!(
            validate_config_value(keys::CORS_ALLOWED_METHODS, r#"["GET", "POST"]"#).is_ok()
        );
        assert!(validate_config_value(
            keys::CORS_ALLOWED_METHODS,
            r#"["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]"#
        )
        .is_ok());
        // 大小写不敏感
        assert!(
            validate_config_value(keys::CORS_ALLOWED_METHODS, r#"["get", "post"]"#).is_ok()
        );
        // 空数组也合法
        assert!(validate_config_value(keys::CORS_ALLOWED_METHODS, r#"[]"#).is_ok());

        // 非法值
        assert!(
            validate_config_value(keys::CORS_ALLOWED_METHODS, r#"["INVALID"]"#).is_err()
        );
        assert!(
            validate_config_value(keys::CORS_ALLOWED_METHODS, r#"["GET", "INVALID"]"#).is_err()
        );
        // 非法 JSON 格式
        assert!(validate_config_value(keys::CORS_ALLOWED_METHODS, "GET,POST").is_err());
        assert!(validate_config_value(keys::CORS_ALLOWED_METHODS, "not json").is_err());
    }

    #[test]
    fn test_get_enum_default_value() {
        assert_eq!(
            get_enum_default_value(keys::API_COOKIE_SAME_SITE),
            Some("Lax".to_string())
        );
        assert!(get_enum_default_value(keys::CORS_ALLOWED_METHODS).is_some());
        assert_eq!(get_enum_default_value("unknown.key"), None);
    }

    #[test]
    fn test_non_enum_config_always_valid() {
        // 非 enum 配置项不做验证，任意值都返回 Ok
        assert!(validate_config_value("some.other.config", "any value").is_ok());
    }
}
