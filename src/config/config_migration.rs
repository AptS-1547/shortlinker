use tracing::{debug, info};

use crate::config::AppConfig;
use crate::errors::Result;
use crate::storage::ConfigStore;

/// 从文件配置迁移到数据库
///
/// 首次启动时，如果数据库中没有配置，则从 config.toml 中读取并写入数据库。
/// 如果数据库中已有配置，则跳过（数据库配置优先）。
pub async fn migrate_config_to_db(file_config: &AppConfig, store: &ConfigStore) -> Result<()> {
    // 检查是否已有配置
    let count = store.count().await?;
    if count > 0 {
        debug!(
            "Database already has {} configuration items, skipping migration",
            count
        );
        return Ok(());
    }

    info!("Starting configuration migration to database");

    // API 认证配置（敏感）
    store
        .insert_if_not_exists(
            "api.admin_token",
            &file_config.api.admin_token,
            "string",
            false,
            true, // is_sensitive
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.health_token",
            &file_config.api.health_token,
            "string",
            false,
            true, // is_sensitive
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.jwt_secret",
            &file_config.api.jwt_secret,
            "string",
            false,
            true, // is_sensitive
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.access_token_minutes",
            &file_config.api.access_token_minutes.to_string(),
            "int",
            false,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.refresh_token_days",
            &file_config.api.refresh_token_days.to_string(),
            "int",
            false,
            false,
        )
        .await?;

    // Cookie 配置（需要重启）
    store
        .insert_if_not_exists(
            "api.access_cookie_name",
            &file_config.api.access_cookie_name,
            "string",
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.refresh_cookie_name",
            &file_config.api.refresh_cookie_name,
            "string",
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.cookie_secure",
            &file_config.api.cookie_secure.to_string(),
            "bool",
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.cookie_same_site",
            &file_config.api.cookie_same_site,
            "string",
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.cookie_domain",
            file_config.api.cookie_domain.as_deref().unwrap_or(""),
            "string",
            true,
            false,
        )
        .await?;

    // 功能配置
    store
        .insert_if_not_exists(
            "features.random_code_length",
            &file_config.features.random_code_length.to_string(),
            "int",
            false,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "features.default_url",
            &file_config.features.default_url,
            "string",
            false,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "features.enable_admin_panel",
            &file_config.features.enable_admin_panel.to_string(),
            "bool",
            true,
            false,
        )
        .await?;

    // 点击统计配置（需要重启 - ClickManager 在启动时创建，配置固化到后台任务）
    store
        .insert_if_not_exists(
            "click.enable_tracking",
            &file_config.click_manager.enable_click_tracking.to_string(),
            "bool",
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "click.flush_interval",
            &file_config.click_manager.flush_interval.to_string(),
            "int",
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "click.max_clicks_before_flush",
            &file_config
                .click_manager
                .max_clicks_before_flush
                .to_string(),
            "int",
            true,
            false,
        )
        .await?;

    // 路由配置（全部需要重启）
    store
        .insert_if_not_exists(
            "routes.admin_prefix",
            &file_config.routes.admin_prefix,
            "string",
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "routes.health_prefix",
            &file_config.routes.health_prefix,
            "string",
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "routes.frontend_prefix",
            &file_config.routes.frontend_prefix,
            "string",
            true,
            false,
        )
        .await?;

    info!("Configuration migration completed successfully");
    Ok(())
}
