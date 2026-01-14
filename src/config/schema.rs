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
    /// 如果是 enum 类型，这里是可选值列表
    #[ts(optional)]
    pub enum_options: Option<Vec<EnumOption>>,
    /// 如果是 JSON 数组类型且每个元素是 enum，这里是可选值列表
    #[ts(optional)]
    pub array_item_options: Option<Vec<EnumOption>>,
    pub requires_restart: bool,
    pub editable: bool,
}

/// 获取所有配置的 schema
pub fn get_all_schemas() -> Vec<ConfigSchema> {
    vec![
        // ========== API 配置 ==========
        ConfigSchema {
            key: keys::API_COOKIE_SAME_SITE.to_string(),
            value_type: ValueType::Enum,
            default_value: SameSitePolicy::default().to_string(),
            description: "Cookie SameSite policy".to_string(),
            enum_options: Some(same_site_options()),
            array_item_options: None,
            requires_restart: true,
            editable: true,
        },
        // ========== CORS 配置 ==========
        ConfigSchema {
            key: keys::CORS_ALLOWED_METHODS.to_string(),
            value_type: ValueType::Json,
            default_value: default_http_methods_json(),
            description: "Allowed HTTP methods for CORS".to_string(),
            enum_options: None,
            array_item_options: Some(http_method_options()),
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
