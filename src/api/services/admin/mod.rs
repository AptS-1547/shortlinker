//! Admin API 服务模块
//!
//! 该模块包含管理 API 的所有端点，包括：
//! - 认证（登录、登出、token 刷新）
//! - 链接 CRUD 操作
//! - 批量操作
//! - 配置管理
//! - 分析统计

pub mod analytics;
pub mod auth;
mod batch_ops;
mod config_ops;
pub mod error_code;
mod export_import;
mod helpers;
mod link_crud;
pub mod routes;
mod types;

// 重新导出类型
pub use types::*;

// 重新导出帮助函数
pub use helpers::{
    api_result, error_from_shortlinker, error_response, parse_expires_at, success_response,
};

// 重新导出错误码
pub use error_code::ErrorCode;

// 重新导出认证端点
pub use auth::{check_admin_token, logout, refresh_token, verify_token};

// 重新导出链接 CRUD 端点
pub use link_crud::{delete_link, get_all_links, get_link, get_stats, post_link, update_link};

// 重新导出批量操作端点
pub use batch_ops::{batch_create_links, batch_delete_links, batch_update_links};

// 重新导出导出导入端点
pub use export_import::{export_links, import_links};

// 重新导出配置管理端点
pub use config_ops::{
    ConfigHistoryResponse, ConfigItemResponse, ConfigUpdateRequest, ConfigUpdateResponse,
    get_all_configs, get_config, get_config_history, get_config_schema, reload_config,
    update_config,
};
