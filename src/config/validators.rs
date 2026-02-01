//! 配置值验证模块
//!
//! 基于 schema 验证配置值是否合法。

use super::ValueType;
use super::schema::get_schema;

/// 根据配置 key 验证值是否合法
///
/// 基于 schema 进行验证：
/// - StringArray: 验证是否为有效的 JSON 字符串数组
/// - EnumArray: 验证 JSON 数组中的每个元素是否在 enum_options 中
/// - Enum: 验证值是否在 enum_options 中
/// - Json + enum_options: 同 EnumArray（向后兼容）
/// - 没有 schema 的配置不做验证
pub fn validate_config_value(key: &str, value: &str) -> Result<(), String> {
    let schema = match get_schema(key) {
        Some(s) => s,
        None => return Ok(()), // 没有 schema 的配置不验证
    };

    // StringArray: 只验证是否为有效的字符串数组
    if schema.value_type == ValueType::StringArray {
        let _items: Vec<String> = serde_json::from_str(value).map_err(|e| {
            format!(
                "Invalid JSON format: {}. Expected array like [\"item1\", \"item2\", ...]",
                e
            )
        })?;
        return Ok(());
    }

    // EnumArray: 验证数组中的每个元素是否在选项中
    if schema.value_type == ValueType::EnumArray {
        let items: Vec<String> = serde_json::from_str(value).map_err(|e| {
            format!(
                "Invalid JSON format: {}. Expected array like [\"GET\", \"POST\", ...]",
                e
            )
        })?;

        if let Some(ref options) = schema.enum_options {
            let valid_values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();
            let mut invalid_items = Vec::new();
            for item in &items {
                if !valid_values.iter().any(|v| v.eq_ignore_ascii_case(item)) {
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
        return Ok(());
    }

    // 有 enum_options 时进行验证（Enum 或 Json + enum_options 向后兼容）
    if let Some(ref options) = schema.enum_options {
        let valid_values: Vec<&str> = options.iter().map(|o| o.value.as_str()).collect();

        // 根据 value_type 决定验证方式
        if schema.value_type == ValueType::Json {
            // JSON 数组类型：验证数组中的每个元素（向后兼容）
            let items: Vec<String> = serde_json::from_str(value).map_err(|e| {
                format!(
                    "Invalid JSON format: {}. Expected array like [\"GET\", \"POST\", ...]",
                    e
                )
            })?;

            let mut invalid_items = Vec::new();
            for item in &items {
                if !valid_values.iter().any(|v| v.eq_ignore_ascii_case(item)) {
                    invalid_items.push(item.clone());
                }
            }

            if !invalid_items.is_empty() {
                return Err(format!(
                    "Invalid items: {:?}. Valid options: {:?}",
                    invalid_items, valid_values
                ));
            }
        } else {
            // 单值类型（Enum）：验证值是否在选项列表中
            if !valid_values.iter().any(|v| v.eq_ignore_ascii_case(value)) {
                return Err(format!(
                    "Invalid value '{}'. Valid options: {:?}",
                    value, valid_values
                ));
            }
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
        assert!(validate_config_value(keys::CORS_ALLOWED_METHODS, r#"["GET", "POST"]"#).is_ok());
        assert!(
            validate_config_value(
                keys::CORS_ALLOWED_METHODS,
                r#"["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"]"#
            )
            .is_ok()
        );
        // 大小写不敏感
        assert!(validate_config_value(keys::CORS_ALLOWED_METHODS, r#"["get", "post"]"#).is_ok());
        // 空数组也合法
        assert!(validate_config_value(keys::CORS_ALLOWED_METHODS, r#"[]"#).is_ok());

        // 非法值
        assert!(validate_config_value(keys::CORS_ALLOWED_METHODS, r#"["INVALID"]"#).is_err());
        assert!(
            validate_config_value(keys::CORS_ALLOWED_METHODS, r#"["GET", "INVALID"]"#).is_err()
        );
        // 非法 JSON 格式
        assert!(validate_config_value(keys::CORS_ALLOWED_METHODS, "GET,POST").is_err());
        assert!(validate_config_value(keys::CORS_ALLOWED_METHODS, "not json").is_err());
    }

    #[test]
    fn test_validate_string_array() {
        // StringArray 类型：只验证是否为有效的 JSON 字符串数组，不验证内容
        // trusted_proxies
        assert!(validate_config_value(keys::API_TRUSTED_PROXIES, r#"["192.168.1.1"]"#).is_ok());
        assert!(
            validate_config_value(
                keys::API_TRUSTED_PROXIES,
                r#"["10.0.0.1", "192.168.1.0/24"]"#
            )
            .is_ok()
        );
        assert!(validate_config_value(keys::API_TRUSTED_PROXIES, r#"[]"#).is_ok());
        // 任意字符串都合法（格式验证由前端负责）
        assert!(validate_config_value(keys::API_TRUSTED_PROXIES, r#"["any", "string"]"#).is_ok());

        // 非法 JSON 格式
        assert!(validate_config_value(keys::API_TRUSTED_PROXIES, "not json").is_err());
        assert!(validate_config_value(keys::API_TRUSTED_PROXIES, r#"{"key": "value"}"#).is_err());

        // allowed_origins
        assert!(
            validate_config_value(keys::CORS_ALLOWED_ORIGINS, r#"["https://example.com"]"#).is_ok()
        );
        assert!(validate_config_value(keys::CORS_ALLOWED_ORIGINS, r#"["*"]"#).is_ok());

        // allowed_headers
        assert!(
            validate_config_value(
                keys::CORS_ALLOWED_HEADERS,
                r#"["Content-Type", "Authorization"]"#
            )
            .is_ok()
        );
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
