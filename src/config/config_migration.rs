use tracing::{debug, info, warn};

use crate::api::services::admin::ValueType;
use crate::config::AppConfig;
use crate::config::runtime_config::keys;
use crate::config::{HttpMethod, SameSitePolicy, default_http_methods_json};
use crate::errors::Result;
use crate::storage::ConfigStore;
use crate::utils::password::{hash_password, is_argon2_hash};

/// 从文件配置迁移到数据库
///
/// 对每个配置项检查是否已存在，如果不存在则从 config.toml 中读取并写入数据库。
/// 这样可以支持新增配置项的增量迁移。
pub async fn migrate_config_to_db(file_config: &AppConfig, store: &ConfigStore) -> Result<()> {
    debug!("Checking configuration migration to database");

    // 检测是否需要打印自动生成的 admin_token
    let admin_token = &file_config.api.admin_token;
    let is_auto_generated = !admin_token.is_empty() && !is_argon2_hash(admin_token);

    // 如果数据库中还没有这个配置，且是自动生成的，打印一次
    if is_auto_generated && store.get(keys::API_ADMIN_TOKEN).await?.is_none() {
        warn!("===========================================");
        warn!("Auto-generated ADMIN_TOKEN: {}", admin_token);
        warn!("Please save this token, it will only be shown once!");
        warn!("===========================================");
    }

    // API 认证配置（敏感）
    store
        .insert_if_not_exists(
            "api.admin_token",
            &file_config.api.admin_token,
            ValueType::String,
            false,
            true, // is_sensitive
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.health_token",
            &file_config.api.health_token,
            ValueType::String,
            false,
            true, // is_sensitive
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.jwt_secret",
            &file_config.api.jwt_secret,
            ValueType::String,
            false,
            true, // is_sensitive
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.access_token_minutes",
            &file_config.api.access_token_minutes.to_string(),
            ValueType::Int,
            false,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.refresh_token_days",
            &file_config.api.refresh_token_days.to_string(),
            ValueType::Int,
            false,
            false,
        )
        .await?;

    // Cookie 配置（需要重启）
    store
        .insert_if_not_exists(
            "api.access_cookie_name",
            &file_config.api.access_cookie_name,
            ValueType::String,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.refresh_cookie_name",
            &file_config.api.refresh_cookie_name,
            ValueType::String,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.cookie_secure",
            &file_config.api.cookie_secure.to_string(),
            ValueType::Bool,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.cookie_same_site",
            &file_config.api.cookie_same_site.to_string(),
            ValueType::Enum,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "api.cookie_domain",
            file_config.api.cookie_domain.as_deref().unwrap_or(""),
            ValueType::String,
            true,
            false,
        )
        .await?;

    // 功能配置
    store
        .insert_if_not_exists(
            "features.random_code_length",
            &file_config.features.random_code_length.to_string(),
            ValueType::Int,
            false,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "features.default_url",
            &file_config.features.default_url,
            ValueType::String,
            false,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "features.enable_admin_panel",
            &file_config.features.enable_admin_panel.to_string(),
            ValueType::Bool,
            true,
            false,
        )
        .await?;

    // 点击统计配置（需要重启 - ClickManager 在启动时创建，配置固化到后台任务）
    store
        .insert_if_not_exists(
            "click.enable_tracking",
            &file_config.click_manager.enable_click_tracking.to_string(),
            ValueType::Bool,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "click.flush_interval",
            &file_config.click_manager.flush_interval.to_string(),
            ValueType::Int,
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
            ValueType::Int,
            true,
            false,
        )
        .await?;

    // 路由配置（全部需要重启）
    store
        .insert_if_not_exists(
            "routes.admin_prefix",
            &file_config.routes.admin_prefix,
            ValueType::String,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "routes.health_prefix",
            &file_config.routes.health_prefix,
            ValueType::String,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "routes.frontend_prefix",
            &file_config.routes.frontend_prefix,
            ValueType::String,
            true,
            false,
        )
        .await?;

    // CORS 配置（需要重启）
    store
        .insert_if_not_exists(
            "cors.enabled",
            &file_config.cors.enabled.to_string(),
            ValueType::Bool,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "cors.allowed_origins",
            &serde_json::to_string(&file_config.cors.allowed_origins).unwrap_or_default(),
            ValueType::Json,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "cors.allowed_methods",
            &serde_json::to_string(&file_config.cors.allowed_methods).unwrap_or_default(),
            ValueType::Json,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "cors.allowed_headers",
            &serde_json::to_string(&file_config.cors.allowed_headers).unwrap_or_default(),
            ValueType::Json,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "cors.max_age",
            &file_config.cors.max_age.to_string(),
            ValueType::Int,
            true,
            false,
        )
        .await?;

    store
        .insert_if_not_exists(
            "cors.allow_credentials",
            &file_config.cors.allow_credentials.to_string(),
            ValueType::Bool,
            true,
            false,
        )
        .await?;

    info!("Configuration migration completed successfully");
    Ok(())
}

/// 自动迁移明文密码到 argon2 哈希
///
/// 检测数据库中的 admin_token，如果是明文则自动哈希并保存。
/// 变更会自动记录到 config_history 表。
pub async fn migrate_plaintext_passwords(store: &ConfigStore) -> Result<()> {
    // 读取 admin_token
    let admin_token = match store.get(keys::API_ADMIN_TOKEN).await? {
        Some(value) => value,
        None => {
            debug!("admin_token not found in database, skipping password migration");
            return Ok(());
        }
    };

    // 如果为空或已是哈希，跳过
    if admin_token.is_empty() {
        debug!("admin_token is empty, skipping password migration");
        return Ok(());
    }

    if is_argon2_hash(&admin_token) {
        debug!("admin_token is already hashed, skipping password migration");
        return Ok(());
    }

    // 明文密码，进行哈希迁移
    info!("Migrating plaintext admin_token to argon2 hash...");

    let hashed = hash_password(&admin_token).map_err(|e| {
        crate::errors::ShortlinkerError::database_operation(format!(
            "Failed to hash admin_token: {}",
            e
        ))
    })?;

    // 保存到数据库（自动记录变更历史）
    store.set(keys::API_ADMIN_TOKEN, &hashed).await?;

    // 同步到 AppConfig
    crate::config::update_config_by_key(keys::API_ADMIN_TOKEN, &hashed);

    warn!(
        "admin_token migrated to argon2 hash successfully. The plaintext password in config.toml is now obsolete."
    );

    Ok(())
}

/// 迁移不合法的 enum 配置值
///
/// 检查数据库中的 enum 类型配置项：
/// 1. 如果 value_type 不是 Enum，则更新为 Enum
/// 2. 如果值不合法，则自动修正为默认值
/// 变更会自动记录到 config_history 表。
pub async fn migrate_enum_configs(store: &ConfigStore) -> Result<()> {
    debug!("Checking enum configuration values for migration");

    // cookie_same_site
    if let Some(item) = store.get_full(keys::API_COOKIE_SAME_SITE).await? {
        // 检查 value_type 是否需要迁移
        if item.value_type != ValueType::Enum {
            info!(
                "Migrating value_type for '{}' from {:?} to Enum",
                keys::API_COOKIE_SAME_SITE, item.value_type
            );
            store
                .update_value_type(keys::API_COOKIE_SAME_SITE, ValueType::Enum)
                .await?;
        }

        // 检查值是否合法
        if item.value.parse::<SameSitePolicy>().is_err() {
            let default = SameSitePolicy::default().to_string();
            warn!(
                "Invalid cookie_same_site '{}', migrating to default '{}'",
                item.value, default
            );
            store.set(keys::API_COOKIE_SAME_SITE, &default).await?;

            // 同步到 AppConfig
            crate::config::update_config_by_key(keys::API_COOKIE_SAME_SITE, &default);
        }
    }

    // 后续扩展其他 enum 配置在这里添加

    // cors.allowed_methods (数组类型 enum)
    if let Some(item) = store.get_full(keys::CORS_ALLOWED_METHODS).await? {
        // 验证 JSON 数组中的每个元素
        let methods: std::result::Result<Vec<String>, _> = serde_json::from_str(&item.value);
        let needs_migration = match methods {
            Ok(methods) => {
                // 检查是否有不合法的方法
                methods
                    .iter()
                    .any(|m| m.parse::<HttpMethod>().is_err())
            }
            Err(_) => true, // JSON 解析失败，需要迁移
        };

        if needs_migration {
            let default = default_http_methods_json();
            warn!(
                "Invalid cors.allowed_methods '{}', migrating to default '{}'",
                item.value, default
            );
            store.set(keys::CORS_ALLOWED_METHODS, &default).await?;

            // 同步到 AppConfig
            crate::config::update_config_by_key(keys::CORS_ALLOWED_METHODS, &default);
        }
    }

    debug!("Enum configuration migration completed");
    Ok(())
}
