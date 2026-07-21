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

use aster_forge_config::{ConfigDefinition, ConfigValueType};

use super::definitions::{CONFIG_REGISTRY, action_for_key, keys};
use super::types::{ActionType, TS_EXPORT_PATH};
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
    /// 排序顺序（基于 Forge registry 中定义的顺序）
    pub order: usize,
    /// 可执行的 action（如生成 token）
    #[ts(optional)]
    pub action: Option<ActionType>,
}

/// 获取所有配置的 schema
///
/// 从 Forge registry 生成，保证存储、校验和展示元数据使用同一份定义。
/// 首次调用会计算并缓存，后续调用直接返回缓存。
pub fn get_all_schemas() -> &'static Vec<ConfigSchema> {
    SCHEMA_CACHE.get_or_init(|| {
        CONFIG_REGISTRY
            .definitions()
            .iter()
            .enumerate()
            .map(|(idx, def)| ConfigSchema {
                key: def.key.to_string(),
                value_type: ValueType::from_forge(def.key, def.value_type),
                default_value: (def.default_fn)(),
                description: def.description.to_string(),
                category: Some(def.category.to_string()),
                enum_options: get_enum_options(def),
                requires_restart: def.requires_restart,
                editable: true,
                order: idx,
                action: action_for_key(def.key),
            })
            .collect()
    })
}

/// 根据 key 获取 schema
pub fn get_schema(key: &str) -> Option<ConfigSchema> {
    get_all_schemas().iter().find(|s| s.key == key).cloned()
}

/// 将产品枚举的候选值附加到 Shortlinker schema。
fn get_enum_options(def: &ConfigDefinition) -> Option<Vec<EnumOption>> {
    match def.key {
        keys::API_COOKIE_SAME_SITE => Some(same_site_options()),
        keys::CORS_ALLOWED_METHODS => Some(http_method_options()),
        keys::ANALYTICS_MAX_ROWS_ACTION => Some(max_rows_action_options()),
        _ if def.value_type == ConfigValueType::Boolean => Some(bool_options()),
        _ => None,
    }
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

fn max_rows_action_options() -> Vec<EnumOption> {
    vec![
        EnumOption {
            value: "cleanup".to_string(),
            label: "Cleanup".to_string(),
            label_i18n_key: Some("enums.maxRowsAction.cleanup.label".to_string()),
            description: Some("Delete oldest records when limit exceeded".to_string()),
            description_i18n_key: Some("enums.maxRowsAction.cleanup.description".to_string()),
        },
        EnumOption {
            value: "stop".to_string(),
            label: "Stop".to_string(),
            label_i18n_key: Some("enums.maxRowsAction.stop.label".to_string()),
            description: Some("Stop logging new clicks when limit exceeded".to_string()),
            description_i18n_key: Some("enums.maxRowsAction.stop.description".to_string()),
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
    fn trusted_proxies_require_restart() {
        let schema = get_schema(keys::API_TRUSTED_PROXIES).expect("trusted proxy schema");

        assert!(schema.requires_restart);
    }

    #[test]
    fn test_get_all_schemas() {
        let schemas = get_all_schemas();
        assert!(!schemas.is_empty());
        assert_eq!(schemas.len(), CONFIG_REGISTRY.definitions().len());
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
        let cfg = ts_rs::Config::default();
        EnumOption::export_all(&cfg).expect("Failed to export EnumOption");
        ConfigSchema::export_all(&cfg).expect("Failed to export ConfigSchema");
        println!("Schema TypeScript types exported to {}", TS_EXPORT_PATH);
    }

    /// 注册表一致性检查：所有 EnumArray 和 Enum 类型必须有 enum_options
    ///
    /// 这个测试确保配置定义的一致性：
    /// - ValueType::Enum 或 EnumArray 必须有对应的 enum_options
    /// - 如果忘记在 get_enum_options() 中处理新的枚举类型，测试会失败
    #[test]
    fn test_enum_types_must_have_options() {
        let schemas = get_all_schemas();
        for schema in schemas {
            if schema.value_type == ValueType::Enum || schema.value_type == ValueType::EnumArray {
                assert!(
                    schema.enum_options.is_some(),
                    "Config '{}' is defined as {:?} but has no enum_options. \
                     Check get_enum_options() in schema.rs to ensure the product enum is handled.",
                    schema.key,
                    schema.value_type
                );
            }
        }
    }
}
