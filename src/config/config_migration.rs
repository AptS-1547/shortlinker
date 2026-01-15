use tracing::{debug, info, warn};

use crate::config::AppConfig;
use crate::config::definitions::{ALL_CONFIGS, keys};
use crate::config::{HttpMethod, SameSitePolicy, default_http_methods_json};
use crate::errors::Result;
use crate::storage::ConfigStore;
use crate::utils::password::{hash_password, is_argon2_hash};

/// 从 AppConfig 中根据 key 获取配置值
///
/// # 维护说明
///
/// 添加新配置时，需要在此函数的 match 中添加对应的映射。
/// 这是目前仍需手动维护的地方之一（另一处在 `impl.rs` 的 `update_config_by_key`）。
///
/// 如果遗漏了某个 key，编译器不会报错，只会在运行时打印警告并返回空字符串。
fn get_config_value(config: &AppConfig, key: &str) -> String {
    match key {
        // API 认证
        keys::API_ADMIN_TOKEN => config.api.admin_token.clone(),
        keys::API_HEALTH_TOKEN => config.api.health_token.clone(),
        keys::API_JWT_SECRET => config.api.jwt_secret.clone(),
        keys::API_ACCESS_TOKEN_MINUTES => config.api.access_token_minutes.to_string(),
        keys::API_REFRESH_TOKEN_DAYS => config.api.refresh_token_days.to_string(),

        // Cookie 配置
        keys::API_ACCESS_COOKIE_NAME => config.api.access_cookie_name.clone(),
        keys::API_REFRESH_COOKIE_NAME => config.api.refresh_cookie_name.clone(),
        keys::API_COOKIE_SECURE => config.api.cookie_secure.to_string(),
        keys::API_COOKIE_SAME_SITE => config.api.cookie_same_site.to_string(),
        keys::API_COOKIE_DOMAIN => config.api.cookie_domain.clone().unwrap_or_default(),

        // 功能配置
        keys::FEATURES_RANDOM_CODE_LENGTH => config.features.random_code_length.to_string(),
        keys::FEATURES_DEFAULT_URL => config.features.default_url.clone(),
        keys::FEATURES_ENABLE_ADMIN_PANEL => config.features.enable_admin_panel.to_string(),

        // 点击统计
        keys::CLICK_ENABLE_TRACKING => config.click_manager.enable_click_tracking.to_string(),
        keys::CLICK_FLUSH_INTERVAL => config.click_manager.flush_interval.to_string(),
        keys::CLICK_MAX_CLICKS_BEFORE_FLUSH => {
            config.click_manager.max_clicks_before_flush.to_string()
        }

        // 路由配置
        keys::ROUTES_ADMIN_PREFIX => config.routes.admin_prefix.clone(),
        keys::ROUTES_HEALTH_PREFIX => config.routes.health_prefix.clone(),
        keys::ROUTES_FRONTEND_PREFIX => config.routes.frontend_prefix.clone(),

        // CORS 配置
        keys::CORS_ENABLED => config.cors.enabled.to_string(),
        keys::CORS_ALLOWED_ORIGINS => {
            serde_json::to_string(&config.cors.allowed_origins).unwrap_or_default()
        }
        keys::CORS_ALLOWED_METHODS => {
            serde_json::to_string(&config.cors.allowed_methods).unwrap_or_default()
        }
        keys::CORS_ALLOWED_HEADERS => {
            serde_json::to_string(&config.cors.allowed_headers).unwrap_or_default()
        }
        keys::CORS_MAX_AGE => config.cors.max_age.to_string(),
        keys::CORS_ALLOW_CREDENTIALS => config.cors.allow_credentials.to_string(),

        // 未知 key，返回空字符串
        _ => {
            warn!("Unknown config key in get_config_value: {}", key);
            String::new()
        }
    }
}

/// 从文件配置迁移到数据库
///
/// 遍历 ALL_CONFIGS 中的所有配置定义，对每个配置项检查是否已存在，
/// 如果不存在则从 config.toml 中读取并写入数据库。
pub async fn migrate_config_to_db(file_config: &AppConfig, store: &ConfigStore) -> Result<()> {
    debug!("Checking configuration migration to database");

    // 特殊处理：检测是否需要打印自动生成的 admin_token
    let admin_token = &file_config.api.admin_token;
    let is_auto_generated = !admin_token.is_empty() && !is_argon2_hash(admin_token);

    if is_auto_generated && store.get(keys::API_ADMIN_TOKEN).await?.is_none() {
        warn!("===========================================");
        warn!("Auto-generated ADMIN_TOKEN: {}", admin_token);
        warn!("Please save this token, it will only be shown once!");
        warn!("===========================================");
    }

    // 遍历所有配置定义进行迁移
    for def in ALL_CONFIGS {
        let value = get_config_value(file_config, def.key);
        let inserted = store
            .insert_if_not_exists(
                def.key,
                &value,
                def.value_type,
                def.requires_restart,
                def.is_sensitive,
            )
            .await?;

        // 兼容旧版本：如果配置已存在，同步元信息（requires_restart / is_sensitive / value_type）
        if !inserted
            && store
                .sync_metadata(
                    def.key,
                    def.value_type,
                    def.requires_restart,
                    def.is_sensitive,
                )
                .await?
        {
            debug!("Synced config metadata for key: {}", def.key);
        }
    }

    info!("Configuration migration completed successfully");
    Ok(())
}

/// 自动迁移明文密码到 argon2 哈希
///
/// 检测数据库中的 admin_token，如果是明文则自动哈希并保存。
/// 变更会自动记录到 config_history 表。
pub async fn migrate_plaintext_passwords(store: &ConfigStore) -> Result<()> {
    let admin_token = match store.get(keys::API_ADMIN_TOKEN).await? {
        Some(value) => value,
        None => {
            debug!("admin_token not found in database, skipping password migration");
            return Ok(());
        }
    };

    if admin_token.is_empty() {
        debug!("admin_token is empty, skipping password migration");
        return Ok(());
    }

    if is_argon2_hash(&admin_token) {
        debug!("admin_token is already hashed, skipping password migration");
        return Ok(());
    }

    info!("Migrating plaintext admin_token to argon2 hash...");

    let hashed = hash_password(&admin_token).map_err(|e| {
        crate::errors::ShortlinkerError::database_operation(format!(
            "Failed to hash admin_token: {}",
            e
        ))
    })?;

    store.set(keys::API_ADMIN_TOKEN, &hashed).await?;
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
pub async fn migrate_enum_configs(store: &ConfigStore) -> Result<()> {
    debug!("Checking enum configuration values for migration");

    // cookie_same_site - 只验证值的合法性，value_type 已由 sync_metadata 同步
    if let Some(item) = store.get_full(keys::API_COOKIE_SAME_SITE).await? {
        if item.value.parse::<SameSitePolicy>().is_err() {
            let default = SameSitePolicy::default().to_string();
            warn!(
                "Invalid cookie_same_site '{}', migrating to default '{}'",
                item.value, default
            );
            store.set(keys::API_COOKIE_SAME_SITE, &default).await?;
            crate::config::update_config_by_key(keys::API_COOKIE_SAME_SITE, &default);
        }
    }

    // cors.allowed_methods (JSON 数组类型 enum)
    if let Some(item) = store.get_full(keys::CORS_ALLOWED_METHODS).await? {
        let methods: std::result::Result<Vec<String>, _> = serde_json::from_str(&item.value);
        let needs_migration = match methods {
            Ok(methods) => methods.iter().any(|m| m.parse::<HttpMethod>().is_err()),
            Err(_) => true,
        };

        if needs_migration {
            let default = default_http_methods_json();
            warn!(
                "Invalid cors.allowed_methods '{}', migrating to default '{}'",
                item.value, default
            );
            store.set(keys::CORS_ALLOWED_METHODS, &default).await?;
            crate::config::update_config_by_key(keys::CORS_ALLOWED_METHODS, &default);
        }
    }

    debug!("Enum configuration migration completed");
    Ok(())
}
