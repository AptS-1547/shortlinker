//! 配置 Schema 定义模块
//!
//! 基于 definitions 中的配置定义生成前端所需的 Schema 信息。
//! 用于：
//! 1. 前端动态渲染配置表单
//! 2. 后端验证配置值
//! 3. 保持前后端同步

use std::sync::OnceLock;

use serde::Serialize;
use strum::IntoEnumIterator;
use ts_rs::TS;

use super::definitions::{ALL_CONFIGS, keys};
use super::types::TS_EXPORT_PATH;
use super::{HttpMethod, SameSitePolicy, ValueType};

/// Schema 缓存
static SCHEMA_CACHE: OnceLock<Vec<ConfigSchema>> = OnceLock::new();

/// 单个 enum 选项
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct EnumOption {
    pub value: String,
    pub label: String,
    #[ts(optional)]
    pub label_i18n_key: Option<String>,
    #[ts(optional)]
    pub description: Option<String>,
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
    #[ts(optional)]
    pub category: Option<String>,
    #[ts(optional)]
    pub enum_options: Option<Vec<EnumOption>>,
    pub requires_restart: bool,
    pub editable: bool,
}

/// 获取所有配置的 schema
///
/// 从 ALL_CONFIGS 生成，保证与配置定义同步。
/// 首次调用会计算并缓存，后续调用直接返回缓存。
pub fn get_all_schemas() -> &'static Vec<ConfigSchema> {
    SCHEMA_CACHE.get_or_init(|| {
        ALL_CONFIGS
            .iter()
            .map(|def| ConfigSchema {
                key: def.key.to_string(),
                value_type: def.value_type,
                default_value: (def.default_fn)(),
                description: def.description.to_string(),
                category: Some(def.category.to_string()),
                enum_options: get_enum_options(def.key),
                requires_restart: def.requires_restart,
                editable: def.editable,
            })
            .collect()
    })
}

/// 根据 key 获取 schema
pub fn get_schema(key: &str) -> Option<ConfigSchema> {
    get_all_schemas().iter().find(|s| s.key == key).cloned()
}

/// 根据 key 获取 enum 选项
fn get_enum_options(key: &str) -> Option<Vec<EnumOption>> {
    match key {
        keys::API_COOKIE_SAME_SITE => Some(same_site_options()),
        keys::CORS_ALLOWED_METHODS => Some(http_method_options()),
        // Bool 类型也提供选项
        k if is_bool_config(k) => Some(bool_options()),
        _ => None,
    }
}

/// 判断是否为 Bool 类型配置
fn is_bool_config(key: &str) -> bool {
    ALL_CONFIGS
        .iter()
        .find(|def| def.key == key)
        .map(|def| def.value_type == ValueType::Bool)
        .unwrap_or(false)
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
        // 应该与 ALL_CONFIGS 数量一致
        assert_eq!(schemas.len(), ALL_CONFIGS.len());
    }

    #[test]
    fn test_same_site_options_type_safe() {
        let options = same_site_options();
        assert_eq!(options.len(), 3);
        for opt in &options {
            assert!(opt.value.parse::<SameSitePolicy>().is_ok());
        }
    }

    #[test]
    fn test_http_method_options_type_safe() {
        let options = http_method_options();
        assert_eq!(options.len(), 7);
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
