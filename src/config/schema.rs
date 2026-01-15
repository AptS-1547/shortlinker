//! 配置 Schema 定义模块
//!
//! 定义所有配置项的元信息，包括类型、默认值、枚举选项等。
//! 用于：
//! 1. 前端动态渲染配置表单
//! 2. 后端验证配置值
//! 3. 保持前后端同步

use serde::Serialize;
use strum::IntoEnumIterator;
use ts_rs::TS;

use super::runtime_config::keys;
use super::{default_http_methods_json, HttpMethod, SameSitePolicy, TS_EXPORT_PATH};
use crate::api::services::admin::ValueType;

/// 单个 enum 选项
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct EnumOption {
    pub value: String,
    pub label: String,
    /// 标签翻译键（如 "enums.sameSite.strict.label"）
    #[ts(optional)]
    pub label_i18n_key: Option<String>,
    /// 描述文本（英文，作为 fallback）
    #[ts(optional)]
    pub description: Option<String>,
    /// 描述翻译键（如 "enums.sameSite.strict.description"）
    #[ts(optional)]
    pub description_i18n_key: Option<String>,
}

/// 配置项的 schema 元信息
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct ConfigSchema {
    pub key: String,
    pub value_type: ValueType,
    pub default_value: String,
    pub description: String,
    /// 配置分组（用于前端分组展示）
    #[ts(optional)]
    pub category: Option<String>,
    /// 枚举选项列表（单选用 Enum，多选用 Json 类型）
    #[ts(optional)]
    pub enum_options: Option<Vec<EnumOption>>,
    pub requires_restart: bool,
    pub editable: bool,
}

/// 配置分组常量
pub mod categories {
    pub const AUTH: &str = "auth";           // 认证配置
    pub const COOKIE: &str = "cookie";       // Cookie 配置
    pub const FEATURES: &str = "features";   // 功能开关
    pub const ROUTES: &str = "routes";       // 路由配置
    pub const CORS: &str = "cors";           // CORS 配置
    pub const TRACKING: &str = "tracking";   // 点击追踪
}

/// 获取所有配置的 schema
pub fn get_all_schemas() -> Vec<ConfigSchema> {
    vec![
        // ========== 认证配置 (auth) ==========
        ConfigSchema {
            key: keys::API_ADMIN_TOKEN.to_string(),
            value_type: ValueType::String,
            default_value: "".to_string(),
            description: "Admin API authentication token (Argon2 hashed)".to_string(),
            category: Some(categories::AUTH.to_string()),
            enum_options: None,
            requires_restart: false,
            editable: true,
        },
        ConfigSchema {
            key: keys::API_HEALTH_TOKEN.to_string(),
            value_type: ValueType::String,
            default_value: "".to_string(),
            description: "Health check endpoint authentication token".to_string(),
            category: Some(categories::AUTH.to_string()),
            enum_options: None,
            requires_restart: false,
            editable: true,
        },
        ConfigSchema {
            key: keys::API_JWT_SECRET.to_string(),
            value_type: ValueType::String,
            default_value: "".to_string(),
            description: "JWT token signing secret key".to_string(),
            category: Some(categories::AUTH.to_string()),
            enum_options: None,
            requires_restart: false,
            editable: true,
        },
        // ========== Cookie 配置 (cookie) ==========
        ConfigSchema {
            key: keys::API_COOKIE_SAME_SITE.to_string(),
            value_type: ValueType::Enum,
            default_value: SameSitePolicy::default().to_string(),
            description: "Cookie SameSite policy".to_string(),
            category: Some(categories::COOKIE.to_string()),
            enum_options: Some(same_site_options()),
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: keys::API_ACCESS_TOKEN_MINUTES.to_string(),
            value_type: ValueType::Int,
            default_value: "60".to_string(),
            description: "Access token expiration time in minutes".to_string(),
            category: Some(categories::COOKIE.to_string()),
            enum_options: None,
            requires_restart: false,
            editable: true,
        },
        ConfigSchema {
            key: keys::API_REFRESH_TOKEN_DAYS.to_string(),
            value_type: ValueType::Int,
            default_value: "7".to_string(),
            description: "Refresh token expiration time in days".to_string(),
            category: Some(categories::COOKIE.to_string()),
            enum_options: None,
            requires_restart: false,
            editable: true,
        },
        ConfigSchema {
            key: "api.access_cookie_name".to_string(),
            value_type: ValueType::String,
            default_value: "access_token".to_string(),
            description: "Access token cookie name".to_string(),
            category: Some(categories::COOKIE.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: "api.refresh_cookie_name".to_string(),
            value_type: ValueType::String,
            default_value: "refresh_token".to_string(),
            description: "Refresh token cookie name".to_string(),
            category: Some(categories::COOKIE.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: "api.cookie_secure".to_string(),
            value_type: ValueType::Bool,
            default_value: "true".to_string(),
            description: "Enable secure flag for cookies (HTTPS only)".to_string(),
            category: Some(categories::COOKIE.to_string()),
            enum_options: Some(bool_options()),
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: "api.cookie_domain".to_string(),
            value_type: ValueType::String,
            default_value: "".to_string(),
            description: "Cookie domain (empty for current domain)".to_string(),
            category: Some(categories::COOKIE.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        // ========== 功能开关 (features) ==========
        ConfigSchema {
            key: keys::FEATURES_RANDOM_CODE_LENGTH.to_string(),
            value_type: ValueType::Int,
            default_value: "6".to_string(),
            description: "Length of randomly generated short codes".to_string(),
            category: Some(categories::FEATURES.to_string()),
            enum_options: None,
            requires_restart: false,
            editable: true,
        },
        ConfigSchema {
            key: keys::FEATURES_DEFAULT_URL.to_string(),
            value_type: ValueType::String,
            default_value: "https://example.com".to_string(),
            description: "Default redirect URL for invalid short codes".to_string(),
            category: Some(categories::FEATURES.to_string()),
            enum_options: None,
            requires_restart: false,
            editable: true,
        },
        ConfigSchema {
            key: keys::FEATURES_ENABLE_ADMIN_PANEL.to_string(),
            value_type: ValueType::Bool,
            default_value: "true".to_string(),
            description: "Enable admin panel interface".to_string(),
            category: Some(categories::FEATURES.to_string()),
            enum_options: Some(bool_options()),
            requires_restart: true,
            editable: true,
        },
        // ========== 路由配置 (routes) ==========
        ConfigSchema {
            key: keys::ROUTES_ADMIN_PREFIX.to_string(),
            value_type: ValueType::String,
            default_value: "/api/admin".to_string(),
            description: "Admin API route prefix".to_string(),
            category: Some(categories::ROUTES.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: keys::ROUTES_HEALTH_PREFIX.to_string(),
            value_type: ValueType::String,
            default_value: "/api/health".to_string(),
            description: "Health check route prefix".to_string(),
            category: Some(categories::ROUTES.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: keys::ROUTES_FRONTEND_PREFIX.to_string(),
            value_type: ValueType::String,
            default_value: "/".to_string(),
            description: "Frontend assets route prefix".to_string(),
            category: Some(categories::ROUTES.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        // ========== CORS 配置 (cors) ==========
        ConfigSchema {
            key: keys::CORS_ENABLED.to_string(),
            value_type: ValueType::Bool,
            default_value: "true".to_string(),
            description: "Enable CORS (Cross-Origin Resource Sharing)".to_string(),
            category: Some(categories::CORS.to_string()),
            enum_options: Some(bool_options()),
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: keys::CORS_ALLOWED_METHODS.to_string(),
            value_type: ValueType::Json,
            default_value: default_http_methods_json(),
            description: "Allowed HTTP methods for CORS".to_string(),
            category: Some(categories::CORS.to_string()),
            enum_options: Some(http_method_options()),
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: keys::CORS_ALLOWED_ORIGINS.to_string(),
            value_type: ValueType::Json,
            default_value: "[\"*\"]".to_string(),
            description: "Allowed origins for CORS (JSON array)".to_string(),
            category: Some(categories::CORS.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: keys::CORS_ALLOWED_HEADERS.to_string(),
            value_type: ValueType::Json,
            default_value: "[\"*\"]".to_string(),
            description: "Allowed headers for CORS (JSON array)".to_string(),
            category: Some(categories::CORS.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: keys::CORS_MAX_AGE.to_string(),
            value_type: ValueType::Int,
            default_value: "3600".to_string(),
            description: "CORS preflight cache duration in seconds".to_string(),
            category: Some(categories::CORS.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: keys::CORS_ALLOW_CREDENTIALS.to_string(),
            value_type: ValueType::Bool,
            default_value: "true".to_string(),
            description: "Allow credentials in CORS requests".to_string(),
            category: Some(categories::CORS.to_string()),
            enum_options: Some(bool_options()),
            requires_restart: true,
            editable: true,
        },
        // ========== 点击追踪 (tracking) ==========
        ConfigSchema {
            key: keys::CLICK_ENABLE_TRACKING.to_string(),
            value_type: ValueType::Bool,
            default_value: "true".to_string(),
            description: "Enable click tracking and analytics".to_string(),
            category: Some(categories::TRACKING.to_string()),
            enum_options: Some(bool_options()),
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: keys::CLICK_FLUSH_INTERVAL.to_string(),
            value_type: ValueType::Int,
            default_value: "60".to_string(),
            description: "Click data flush interval in seconds".to_string(),
            category: Some(categories::TRACKING.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
        ConfigSchema {
            key: "click.max_clicks_before_flush".to_string(),
            value_type: ValueType::Int,
            default_value: "1000".to_string(),
            description: "Maximum clicks before forcing flush".to_string(),
            category: Some(categories::TRACKING.to_string()),
            enum_options: None,
            requires_restart: true,
            editable: true,
        },
    ]
}

/// 根据 key 获取 schema
pub fn get_schema(key: &str) -> Option<ConfigSchema> {
    get_all_schemas().into_iter().find(|s| s.key == key)
}

// ========== enum 选项定义（类型安全，自动生成） ==========

fn same_site_options() -> Vec<EnumOption> {
    use strum::EnumMessage;

    SameSitePolicy::iter()
        .map(|v| {
            let value = v.as_ref().to_string();
            let value_lower = value.to_lowercase();
            EnumOption {
                value: value.clone(),
                label: value.clone(),
                label_i18n_key: Some(format!("enums.sameSite.{}.label", value_lower)),
                description: v.get_message().map(|s| s.to_string()),
                description_i18n_key: Some(format!("enums.sameSite.{}.description", value_lower)),
            }
        })
        .collect()
}

fn http_method_options() -> Vec<EnumOption> {
    HttpMethod::iter()
        .map(|v| {
            let value = v.as_ref().to_string();
            let value_lower = value.to_lowercase();
            EnumOption {
                value: value.clone(),
                label: value.clone(),
                label_i18n_key: Some(format!("enums.httpMethod.{}.label", value_lower)),
                description: None,
                description_i18n_key: Some(format!("enums.httpMethod.{}.description", value_lower)),
            }
        })
        .collect()
}

/// Bool 类型的标准 enum 选项
fn bool_options() -> Vec<EnumOption> {
    vec![
        EnumOption {
            value: "true".to_string(),
            label: "Enabled".to_string(),
            label_i18n_key: Some("common.enabled".to_string()),
            description: None,
            description_i18n_key: None,
        },
        EnumOption {
            value: "false".to_string(),
            label: "Disabled".to_string(),
            label_i18n_key: Some("common.disabled".to_string()),
            description: None,
            description_i18n_key: None,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_schema() {
        let schema = get_schema(keys::API_COOKIE_SAME_SITE);
        assert!(schema.is_some());
        let schema = schema.unwrap();
        assert_eq!(schema.value_type, ValueType::Enum);
        assert!(schema.enum_options.is_some());
    }

    #[test]
    fn test_get_all_schemas() {
        let schemas = get_all_schemas();
        assert!(!schemas.is_empty());
    }

    #[test]
    fn test_same_site_options_type_safe() {
        // 验证 enum 选项与实际 enum 定义一致
        let options = same_site_options();
        assert_eq!(options.len(), 3);

        // 验证所有选项都能被 FromStr 解析
        for opt in &options {
            assert!(opt.value.parse::<SameSitePolicy>().is_ok());
        }
    }

    #[test]
    fn test_http_method_options_type_safe() {
        // 验证 enum 选项与实际 enum 定义一致
        let options = http_method_options();
        assert_eq!(options.len(), 7);

        // 验证所有选项都能被 FromStr 解析
        for opt in &options {
            assert!(opt.value.parse::<HttpMethod>().is_ok());
        }
    }

    #[test]
    fn export_typescript_types() {
        EnumOption::export_all().expect("Failed to export EnumOption");
        ConfigSchema::export_all().expect("Failed to export ConfigSchema");
        println!("Schema TypeScript types exported to {}", TS_EXPORT_PATH);
    }
}
