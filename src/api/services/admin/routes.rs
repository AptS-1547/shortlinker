//! Admin API 路由配置
//!
//! 将 /v1 下的路由按功能模块拆分，提高可读性和可维护性。

use actix_web::web;

use super::analytics::{analytics_routes, get_link_analytics, get_link_device_stats};
use super::auth::{
    check_admin_token, login_rate_limiter, logout, refresh_rate_limiter, refresh_token,
    verify_token,
};
use super::batch_ops::{batch_create_links, batch_delete_links, batch_update_links};
use super::config_ops::{
    execute_and_save_config_action, execute_config_action, get_all_configs, get_config,
    get_config_history, get_config_schema, reload_config, update_config,
};
use super::export_import::{export_links, import_links};
use super::link_crud::{delete_link, get_all_links, get_link, get_stats, post_link, update_link};

/// 链接管理路由 `/links`
///
/// 包含：
/// - GET/HEAD /links - 获取所有链接
/// - POST /links - 创建链接
/// - GET/HEAD /links/{code} - 获取单个链接
/// - PUT /links/{code} - 更新链接
/// - DELETE /links/{code} - 删除链接
/// - GET /links/{code}/analytics - 获取单链接统计
/// - GET /links/{code}/analytics/devices - 获取单链接设备统计
pub fn links_routes() -> actix_web::Scope {
    web::scope("/links")
        .route("", web::get().to(get_all_links))
        .route("", web::head().to(get_all_links))
        .route("", web::post().to(post_link))
        // Batch operations (must be before /{code:.*})
        .route("/batch", web::post().to(batch_create_links))
        .route("/batch", web::put().to(batch_update_links))
        .route("/batch", web::delete().to(batch_delete_links))
        // Export/Import operations (must be before /{code:.*})
        .route("/export", web::get().to(export_links))
        .route("/import", web::post().to(import_links))
        // Single link analytics (must be before /{code:.*})
        .route(
            "/{code}/analytics/devices",
            web::get().to(get_link_device_stats),
        )
        .route("/{code}/analytics", web::get().to(get_link_analytics))
        // Single link operations (must be last due to wildcard)
        .route("/{code:.*}", web::get().to(get_link))
        .route("/{code:.*}", web::head().to(get_link))
        .route("/{code:.*}", web::put().to(update_link))
        .route("/{code:.*}", web::delete().to(delete_link))
}

/// 统计路由 `/stats`
pub fn stats_routes() -> actix_web::Scope {
    web::scope("/stats")
        .route("", web::get().to(get_stats))
        .route("", web::head().to(get_stats))
}

/// 认证路由 `/auth`
///
/// 包含：
/// - POST /auth/login - 登录（带限流）
/// - POST /auth/refresh - 刷新 token（带限流）
/// - POST /auth/logout - 登出
/// - GET /auth/verify - 验证 token
pub fn auth_routes() -> actix_web::Scope {
    web::scope("/auth")
        .route(
            "/login",
            web::post().to(check_admin_token).wrap(login_rate_limiter()),
        )
        .route(
            "/refresh",
            web::post().to(refresh_token).wrap(refresh_rate_limiter()),
        )
        .route("/logout", web::post().to(logout))
        .route("/verify", web::get().to(verify_token))
}

/// 配置管理路由 `/config`
///
/// 包含：
/// - GET /config - 获取所有配置
/// - POST /config/reload - 重载配置
/// - GET /config/schema - 获取配置 schema
/// - POST /config/{key}/action - 执行配置 action（如生成 token）
/// - POST /config/{key}/execute-and-save - 执行 action 并保存（安全版本）
/// - GET /config/{key}/history - 获取配置历史
/// - GET /config/{key} - 获取单个配置
/// - PUT /config/{key} - 更新配置
pub fn config_routes() -> actix_web::Scope {
    web::scope("/config")
        .route("", web::get().to(get_all_configs))
        .route("/reload", web::post().to(reload_config))
        .route("/schema", web::get().to(get_config_schema))
        // {key:.*}/execute-and-save must be before {key:.*}
        .route(
            "/{key:.*}/execute-and-save",
            web::post().to(execute_and_save_config_action),
        )
        // {key:.*}/action must be before {key:.*}
        .route("/{key:.*}/action", web::post().to(execute_config_action))
        // {key:.*}/history must be before {key:.*}
        .route("/{key:.*}/history", web::get().to(get_config_history))
        .route("/{key:.*}", web::get().to(get_config))
        .route("/{key:.*}", web::put().to(update_config))
}

/// Admin API v1 路由
///
/// 组合所有子模块路由
pub fn admin_v1_routes() -> actix_web::Scope {
    web::scope("/v1")
        .service(links_routes())
        .service(stats_routes())
        .service(auth_routes())
        .service(config_routes())
        .service(analytics_routes())
}
