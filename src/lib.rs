//! Shortlinker - A high-performance URL shortener service
//!
//! This library provides the core functionality for the Shortlinker service,
//! including caching, storage backends, HTTP services, and management interfaces.
//!
//! # Features
//! - **server**: HTTP server mode (default)
//! - **cli**: Command-line interface
//! - **metrics**: Prometheus metrics export
//! - **full**: All features enabled
//!
//! # Architecture
//! - `services`: Link cache policy and product business logic
//! - `storage`: Storage backends and data access
//! - `analytics`: Click tracking and statistics
//! - `api`: HTTP services and middleware
//! - `interfaces`: Command-line interface
//! - `config`: Configuration management
//! - `runtime`: Application lifecycle and execution modes
//! - `system`: Platform abstraction and system utilities

/// 点击事件缓冲、落盘、小时/每日汇总与保留策略。
pub mod analytics;
/// HTTP 路由、管理 API、认证/CSRF/限流中间件与重定向端点。
pub mod api;
/// 命令行入口：链接管理、导入导出、配置与状态子命令。
pub mod cli;
/// 内部 IPC 客户端与链接/配置/系统服务的客户端封装。
pub mod client;
/// 静态配置、运行时配置、schema、校验与默认值。
pub mod config;
/// 领域错误类型与统一错误码转换。
pub mod errors;
/// Prometheus 指标抽象（`metrics` feature）。
pub mod metrics;
/// 启动、关闭、生命周期与 server mode 装配。
pub mod runtime;
/// 链接、配置、分析、GeoIP 等业务编排服务。
pub mod services;
/// SeaORM 存储实现、数据库连接与查询/写入后端。
pub mod storage;
/// 日志、panic、平台差异与进程相关系统工具。
pub mod system;
/// URL、短码、密码、时间解析等通用工具函数。
pub mod utils;
