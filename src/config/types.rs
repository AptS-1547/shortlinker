//! 配置类型定义模块
//!
//! 定义 Shortlinker 配置 API 的展示类型。

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 输出目录常量
pub const TS_EXPORT_PATH: &str = "../admin-panel/src/services/types.generated.ts";

/// 配置 Action 类型枚举
///
/// 用于标识配置项可执行的操作（如生成 token）。
/// 这是一个与 ValueType 正交的概念 - 任何类型的配置都可以有可选的 action。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// 生成安全随机 token（32 字节 hex）
    GenerateToken,
}

/// 配置值类型枚举
///
/// 用于标识配置项在数据库和前端的类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
#[serde(rename_all = "lowercase")]
pub enum ValueType {
    String,
    Int,
    /// 浮点数类型，前端渲染为数字输入（step=0.01）
    Float,
    Bool,
    Json,
    Enum,
    /// 字符串数组类型，前端渲染为 Tag Input
    StringArray,
    /// 枚举数组类型，前端渲染为多选 Checkbox（需配合 enum_options）
    EnumArray,
}

impl ValueType {
    /// 将 Forge 的存储类型映射为 Shortlinker 已发布的 API/TypeScript 展示类型。
    ///
    /// Forge 统一把数字存储为 `number`；Shortlinker 的管理界面仍区分整数和
    /// 点击采样率浮点数，这是产品协议边界而不是第二套配置定义。
    pub fn from_forge(key: &str, value_type: aster_forge_config::ConfigValueType) -> Self {
        use aster_forge_config::ConfigValueType;

        match value_type {
            ConfigValueType::String | ConfigValueType::Multiline => Self::String,
            ConfigValueType::StringArray => Self::StringArray,
            ConfigValueType::StringEnum => Self::Enum,
            ConfigValueType::StringEnumSet => Self::EnumArray,
            ConfigValueType::Number if key == super::keys::ANALYTICS_SAMPLE_RATE => Self::Float,
            ConfigValueType::Number => Self::Int,
            ConfigValueType::Boolean => Self::Bool,
        }
    }
}

impl std::fmt::Display for ValueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Int => write!(f, "int"),
            Self::Float => write!(f, "float"),
            Self::Bool => write!(f, "bool"),
            Self::Json => write!(f, "json"),
            Self::Enum => write!(f, "enum"),
            Self::StringArray => write!(f, "stringarray"),
            Self::EnumArray => write!(f, "enumarray"),
        }
    }
}

impl std::str::FromStr for ValueType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" => Ok(Self::String),
            "multiline" => Ok(Self::String),
            "int" | "number" => Ok(Self::Int),
            "float" => Ok(Self::Float),
            "bool" | "boolean" => Ok(Self::Bool),
            "json" => Ok(Self::Json),
            "enum" | "string_enum" => Ok(Self::Enum),
            "stringarray" | "string_array" => Ok(Self::StringArray),
            "enumarray" | "string_enum_set" => Ok(Self::EnumArray),
            _ => Err(format!("Unknown value type: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_type_display() {
        assert_eq!(ValueType::String.to_string(), "string");
        assert_eq!(ValueType::Int.to_string(), "int");
        assert_eq!(ValueType::Float.to_string(), "float");
        assert_eq!(ValueType::Bool.to_string(), "bool");
        assert_eq!(ValueType::Json.to_string(), "json");
        assert_eq!(ValueType::Enum.to_string(), "enum");
        assert_eq!(ValueType::StringArray.to_string(), "stringarray");
        assert_eq!(ValueType::EnumArray.to_string(), "enumarray");
    }

    #[test]
    fn test_value_type_from_str() {
        assert_eq!("string".parse::<ValueType>().unwrap(), ValueType::String);
        assert_eq!("int".parse::<ValueType>().unwrap(), ValueType::Int);
        assert_eq!("float".parse::<ValueType>().unwrap(), ValueType::Float);
        assert_eq!("bool".parse::<ValueType>().unwrap(), ValueType::Bool);
        assert_eq!("json".parse::<ValueType>().unwrap(), ValueType::Json);
        assert_eq!("enum".parse::<ValueType>().unwrap(), ValueType::Enum);
        assert_eq!(
            "stringarray".parse::<ValueType>().unwrap(),
            ValueType::StringArray
        );
        assert_eq!(
            "enumarray".parse::<ValueType>().unwrap(),
            ValueType::EnumArray
        );
        assert!("invalid".parse::<ValueType>().is_err());
    }

    #[test]
    fn forge_storage_types_preserve_shortlinker_wire_types() {
        use aster_forge_config::ConfigValueType;

        assert_eq!(
            ValueType::from_forge(
                super::super::keys::ANALYTICS_SAMPLE_RATE,
                ConfigValueType::Number
            ),
            ValueType::Float
        );
        assert_eq!(
            ValueType::from_forge("cors.max_age", ConfigValueType::Number),
            ValueType::Int
        );
        assert_eq!(
            ValueType::from_forge("cors.allowed_methods", ConfigValueType::StringEnumSet),
            ValueType::EnumArray
        );
    }

    #[test]
    fn export_typescript_types() {
        let cfg = ts_rs::Config::default();
        ActionType::export_all(&cfg).expect("Failed to export ActionType");
        ValueType::export_all(&cfg).expect("Failed to export ValueType");
        println!("ActionType and ValueType exported to {}", TS_EXPORT_PATH);
    }
}
