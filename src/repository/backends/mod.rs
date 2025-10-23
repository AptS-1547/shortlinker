pub mod sea_orm;

use crate::errors::{Result, ShortlinkerError};

/// 从数据库 URL 推断数据库类型
pub fn infer_backend_from_url(database_url: &str) -> Result<String> {
    if database_url.starts_with("sqlite://")
        || database_url.ends_with(".db")
        || database_url.ends_with(".sqlite")
        || database_url == ":memory:"
    {
        Ok("sqlite".to_string())
    } else if database_url.starts_with("mysql://") {
        Ok("mysql".to_string())
    } else if database_url.starts_with("mariadb://") {
        Ok("mysql".to_string()) // MariaDB 使用 MySQL 协议
    } else if database_url.starts_with("postgres://") || database_url.starts_with("postgresql://") {
        Ok("postgres".to_string())
    } else {
        Err(ShortlinkerError::database_config(format!(
            "无法从 URL 推断数据库类型: {}. 支持的 URL 格式: sqlite://, mysql://, mariadb://, postgres://",
            database_url
        )))
    }
}

/// 规范化 backend 名称
pub fn normalize_backend_name(backend: &str) -> String {
    match backend {
        "mariadb" => "mysql".to_string(),
        other => other.to_string(),
    }
}
