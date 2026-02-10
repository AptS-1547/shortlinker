//! Service layer for business logic
//!
//! This module provides unified business logic that can be shared between
//! different interfaces (HTTP API, IPC, CLI).
//!
//! # Service 优先原则
//!
//! 所有 admin API handler 必须通过 service 层访问数据，不得直接调用 Storage。
//!
//! ## 例外（已文档化）
//! - `redirect` handler：热路径，直连 Storage + Cache（见 `api/services/redirect.rs`）
//! - `health` handler：基础设施路径，直连 Storage + Cache（见 `api/services/health.rs`）
//!
//! ## Service 清单
//! - [`LinkService`]：链接 CRUD、批量操作、导入导出
//! - [`AnalyticsService`]：点击分析、趋势、导出
//! - [`ConfigService`]：运行时配置管理

mod analytics_service;
mod config_service;
pub mod geoip;
pub mod import_validation;
mod link_service;
mod user_agent_store;

pub use analytics_service::*;
pub use config_service::*;
pub use geoip::{GeoInfo, GeoIpLookup, GeoIpProvider};
pub use import_validation::{
    ImportLinkItemRaw, ImportRowError, validate_import_row, validate_import_rows,
};
pub use link_service::*;
pub use user_agent_store::{UserAgentStore, get_user_agent_store, set_global_user_agent_store};
