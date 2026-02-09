//! 配置类型定义模块
//!
//! 定义配置系统的核心类型，包括值类型、Rust 类型标识等。

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
            "int" => Ok(Self::Int),
            "float" => Ok(Self::Float),
            "bool" => Ok(Self::Bool),
            "json" => Ok(Self::Json),
            "enum" => Ok(Self::Enum),
            "stringarray" => Ok(Self::StringArray),
            "enumarray" => Ok(Self::EnumArray),
            _ => Err(format!("Unknown value type: {}", s)),
        }
    }
}

/// Rust 类型标识
///
/// 用于类型安全的配置更新，标识配置项在 Rust 代码中的实际类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RustType {
    /// String 类型
    String,
    /// u64 类型
    U64,
    /// usize 类型
    Usize,
    /// f64 类型
    F64,
    /// bool 类型
    Bool,
    /// Option<String> 类型
    OptionString,
    /// Vec<String> 类型
    VecString,
    /// Vec<HttpMethod> 类型
    VecHttpMethod,
    /// SameSitePolicy 枚举类型
    SameSitePolicy,
    /// MaxRowsAction 枚举类型（cleanup 或 stop）
    MaxRowsAction,
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
    fn test_rust_type_variants() {
        // 确保所有 RustType 变体都可以创建
        let variants = [
            RustType::String,
            RustType::U64,
            RustType::Usize,
            RustType::F64,
            RustType::Bool,
            RustType::OptionString,
            RustType::VecString,
            RustType::VecHttpMethod,
            RustType::SameSitePolicy,
            RustType::MaxRowsAction,
        ];
        assert_eq!(variants.len(), 10);
    }

    #[test]
    fn export_typescript_types() {
        let cfg = ts_rs::Config::default();
        ActionType::export_all(&cfg).expect("Failed to export ActionType");
        ValueType::export_all(&cfg).expect("Failed to export ValueType");
        println!("ActionType and ValueType exported to {}", TS_EXPORT_PATH);
    }
}
