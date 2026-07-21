//! Shortlinker 运行时配置注册表。
//!
//! 配置 key、存储类型、默认值和后端元数据统一注册到 Forge
//! [`aster_forge_config::ConfigRegistry`]；Shortlinker 只保留短链接产品特有的
//! 值归一化规则和配置 action。
//!
//! # 添加新配置的步骤
//!
//! 1. 在 `keys` 模块中添加新的 key 常量
//! 2. 添加默认值函数（如果需要动态默认值）
//! 3. 在 [`CONFIG_REGISTRY`] 中添加 Forge `ConfigDefinition`

use std::str::FromStr;

use aster_forge_config::{
    ConfigCoreError, ConfigDefinition, ConfigValueLookup, ConfigValueType,
    normalize_non_negative_u64_config_value, normalize_positive_u64_config_value,
    normalize_string_enum_set_selection, parse_single_string_enum_selection,
    parse_string_array_config_value,
};

use super::types::ActionType;
use super::{HttpMethod, SameSitePolicy};

/// 配置分类常量
pub mod categories {
    pub const AUTH: &str = "auth";
    pub const COOKIE: &str = "cookie";
    pub const FEATURES: &str = "features";
    pub const ROUTES: &str = "routes";
    pub const CORS: &str = "cors";
    pub const TRACKING: &str = "tracking";
    pub const ANALYTICS: &str = "analytics";
    pub const CACHE: &str = "cache";
}

/// Key 常量
pub mod keys {
    // API 认证
    pub const API_ADMIN_TOKEN: &str = "api.admin_token";
    pub const API_HEALTH_TOKEN: &str = "api.health_token";
    pub const API_JWT_SECRET: &str = "api.jwt_secret";
    pub const API_ACCESS_TOKEN_MINUTES: &str = "api.access_token_minutes";
    pub const API_REFRESH_TOKEN_DAYS: &str = "api.refresh_token_days";
    pub const API_TRUSTED_PROXIES: &str = "api.trusted_proxies";

    // Cookie 配置
    pub const API_COOKIE_SECURE: &str = "api.cookie_secure";
    pub const API_COOKIE_SAME_SITE: &str = "api.cookie_same_site";
    pub const API_COOKIE_DOMAIN: &str = "api.cookie_domain";

    // 功能配置
    pub const FEATURES_RANDOM_CODE_LENGTH: &str = "features.random_code_length";
    pub const FEATURES_DEFAULT_URL: &str = "features.default_url";
    pub const FEATURES_ENABLE_ADMIN_PANEL: &str = "features.enable_admin_panel";

    // 点击统计
    pub const CLICK_ENABLE_TRACKING: &str = "click.enable_tracking";
    pub const CLICK_FLUSH_INTERVAL: &str = "click.flush_interval";
    pub const CLICK_MAX_CLICKS_BEFORE_FLUSH: &str = "click.max_clicks_before_flush";

    // 详细分析统计
    pub const ANALYTICS_ENABLE_DETAILED_LOGGING: &str = "analytics.enable_detailed_logging";
    pub const ANALYTICS_LOG_RETENTION_DAYS: &str = "analytics.log_retention_days";
    pub const ANALYTICS_ENABLE_IP_LOGGING: &str = "analytics.enable_ip_logging";
    pub const ANALYTICS_ENABLE_GEO_LOOKUP: &str = "analytics.enable_geo_lookup";
    pub const ANALYTICS_HOURLY_RETENTION_DAYS: &str = "analytics.hourly_retention_days";
    pub const ANALYTICS_DAILY_RETENTION_DAYS: &str = "analytics.daily_retention_days";
    pub const ANALYTICS_ENABLE_AUTO_ROLLUP: &str = "analytics.enable_auto_rollup";
    pub const ANALYTICS_SAMPLE_RATE: &str = "analytics.sample_rate";
    pub const ANALYTICS_MAX_LOG_ROWS: &str = "analytics.max_log_rows";
    pub const ANALYTICS_MAX_ROWS_ACTION: &str = "analytics.max_rows_action";

    // UTM 追踪
    pub const UTM_ENABLE_PASSTHROUGH: &str = "utm.enable_passthrough";

    // 路由配置
    pub const ROUTES_ADMIN_PREFIX: &str = "routes.admin_prefix";
    pub const ROUTES_HEALTH_PREFIX: &str = "routes.health_prefix";
    pub const ROUTES_FRONTEND_PREFIX: &str = "routes.frontend_prefix";

    // CORS 配置
    pub const CORS_ENABLED: &str = "cors.enabled";
    pub const CORS_ALLOWED_ORIGINS: &str = "cors.allowed_origins";
    pub const CORS_ALLOWED_METHODS: &str = "cors.allowed_methods";
    pub const CORS_ALLOWED_HEADERS: &str = "cors.allowed_headers";
    pub const CORS_MAX_AGE: &str = "cors.max_age";
    pub const CORS_ALLOW_CREDENTIALS: &str = "cors.allow_credentials";

    // 缓存配置
    pub const CACHE_BLOOM_REBUILD_INTERVAL: &str = "cache.bloom_rebuild_interval";
}

// 默认值函数
fn default_empty() -> String {
    String::new()
}

fn default_admin_token() -> String {
    String::new() // 默认为空，用户需运行 reset-password 手动设置
}

fn default_jwt_secret() -> String {
    crate::utils::generate_secure_token(32)
}

fn default_access_token_minutes() -> String {
    "15".to_string()
}

fn default_refresh_token_days() -> String {
    "7".to_string()
}

fn default_cookie_secure() -> String {
    "true".to_string()
}

fn default_cookie_same_site() -> String {
    "Lax".to_string()
}

fn default_trusted_proxies() -> String {
    "[]".to_string()
}

fn default_random_code_length() -> String {
    "6".to_string()
}

fn default_default_url() -> String {
    "https://esap.cc/repo".to_string()
}

fn default_enable_admin_panel() -> String {
    "false".to_string()
}

fn default_enable_tracking() -> String {
    "true".to_string()
}

fn default_flush_interval() -> String {
    "30".to_string()
}

fn default_max_clicks_before_flush() -> String {
    "100".to_string()
}

fn default_admin_prefix() -> String {
    "/admin".to_string()
}

fn default_health_prefix() -> String {
    "/health".to_string()
}

fn default_frontend_prefix() -> String {
    "/panel".to_string()
}

fn default_cors_enabled() -> String {
    "false".to_string()
}

fn default_cors_allowed_origins() -> String {
    "[]".to_string()
}

fn default_cors_allowed_methods() -> String {
    crate::config::default_http_methods_json()
}

fn default_cors_allowed_headers() -> String {
    r#"["Content-Type","Authorization","Accept"]"#.to_string()
}

fn default_cors_max_age() -> String {
    "3600".to_string()
}

fn default_cors_allow_credentials() -> String {
    "false".to_string()
}

fn default_analytics_enable_detailed_logging() -> String {
    "false".to_string()
}

fn default_analytics_log_retention_days() -> String {
    "30".to_string()
}

fn default_analytics_enable_ip_logging() -> String {
    "true".to_string()
}

fn default_analytics_enable_geo_lookup() -> String {
    "false".to_string()
}

fn default_analytics_hourly_retention_days() -> String {
    "7".to_string()
}

fn default_analytics_daily_retention_days() -> String {
    "365".to_string()
}

fn default_analytics_enable_auto_rollup() -> String {
    "true".to_string()
}

fn default_analytics_sample_rate() -> String {
    "1.0".to_string() // 默认记录所有点击
}

fn default_analytics_max_log_rows() -> String {
    "0".to_string() // 默认不限制
}

fn default_analytics_max_rows_action() -> String {
    "cleanup".to_string() // 默认自动清理
}

fn default_utm_enable_passthrough() -> String {
    "false".to_string()
}

fn default_bloom_rebuild_interval() -> String {
    "14400".to_string() // 4 hours, 0 = disabled
}

fn normalize_trusted_proxies(
    _lookup: &dyn ConfigValueLookup,
    key: &str,
    value: &str,
) -> aster_forge_config::Result<String> {
    let proxies = parse_string_array_config_value(value, key)?;
    if aster_forge_utils::net::parse_trusted_proxies(&proxies).len() != proxies.len() {
        return Err(ConfigCoreError::invalid_value(
            "trusted proxies must contain only valid IP addresses or CIDRs",
        ));
    }
    serde_json::to_string(&proxies).map_err(Into::into)
}

fn normalize_same_site(
    _lookup: &dyn ConfigValueLookup,
    key: &str,
    value: &str,
) -> aster_forge_config::Result<String> {
    let policy = parse_single_string_enum_selection(value, key, "Strict, Lax, None", |raw| {
        SameSitePolicy::from_str(raw).ok()
    })?;
    Ok(policy.to_string())
}

fn normalize_http_methods(
    _lookup: &dyn ConfigValueLookup,
    key: &str,
    value: &str,
) -> aster_forge_config::Result<String> {
    const METHODS: &[HttpMethod] = &[
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
        HttpMethod::Patch,
        HttpMethod::Head,
        HttpMethod::Options,
    ];

    let methods = normalize_string_enum_set_selection(
        value,
        key,
        "HTTP method",
        METHODS,
        |raw| HttpMethod::from_str(raw).ok(),
        |method| match method {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        },
    )?;
    serde_json::to_string(&methods).map_err(Into::into)
}

fn normalize_max_rows_action(
    _lookup: &dyn ConfigValueLookup,
    key: &str,
    value: &str,
) -> aster_forge_config::Result<String> {
    parse_single_string_enum_selection(value, key, "cleanup, stop", |raw| {
        match raw.to_ascii_lowercase().as_str() {
            "cleanup" => Some("cleanup"),
            "stop" => Some("stop"),
            _ => None,
        }
    })
    .map(str::to_string)
}

fn normalize_unsigned_integer(
    _lookup: &dyn ConfigValueLookup,
    key: &str,
    value: &str,
) -> aster_forge_config::Result<String> {
    match key {
        keys::API_ACCESS_TOKEN_MINUTES
        | keys::API_REFRESH_TOKEN_DAYS
        | keys::FEATURES_RANDOM_CODE_LENGTH
        | keys::CLICK_FLUSH_INTERVAL
        | keys::CLICK_MAX_CLICKS_BEFORE_FLUSH => normalize_positive_u64_config_value(key, value),
        keys::CORS_MAX_AGE
        | keys::ANALYTICS_LOG_RETENTION_DAYS
        | keys::ANALYTICS_HOURLY_RETENTION_DAYS
        | keys::ANALYTICS_DAILY_RETENTION_DAYS
        | keys::ANALYTICS_MAX_LOG_ROWS
        | keys::CACHE_BLOOM_REBUILD_INTERVAL => normalize_non_negative_u64_config_value(key, value),
        _ => Err(ConfigCoreError::invalid_value(format!(
            "'{key}' is not an unsigned-integer configuration"
        ))),
    }
}

fn normalize_sample_rate(
    _lookup: &dyn ConfigValueLookup,
    key: &str,
    value: &str,
) -> aster_forge_config::Result<String> {
    let sample_rate = value.trim().parse::<f64>().map_err(|_| {
        ConfigCoreError::invalid_value(format!("{key} must be a number between 0.0 and 1.0"))
    })?;
    if !sample_rate.is_finite() || !(0.0..=1.0).contains(&sample_rate) {
        return Err(ConfigCoreError::invalid_value(format!(
            "{key} must be a finite number between 0.0 and 1.0"
        )));
    }
    if sample_rate.fract() == 0.0 {
        Ok(format!("{sample_rate:.1}"))
    } else {
        Ok(sample_rate.to_string())
    }
}

aster_forge_config::define_config_registry! {
pub static CONFIG_REGISTRY = [
    // ========== API 认证 (auth) ==========
    ConfigDefinition {
        key: keys::API_ADMIN_TOKEN,
        label_i18n_key: "config.keys.api.admin_token",
        description_i18n_key: "config.descriptions.api.admin_token",
        value_type: ConfigValueType::String,
        default_fn: default_admin_token,
        requires_restart: false,
        is_sensitive: true,
        category: categories::AUTH,
        description: "Admin API authentication token (Argon2 hashed)",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::API_HEALTH_TOKEN,
        label_i18n_key: "config.keys.api.health_token",
        description_i18n_key: "config.descriptions.api.health_token",
        value_type: ConfigValueType::String,
        default_fn: default_empty,
        is_sensitive: true,
        category: categories::AUTH,
        description: "Health check endpoint authentication token",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::API_JWT_SECRET,
        label_i18n_key: "config.keys.api.jwt_secret",
        description_i18n_key: "config.descriptions.api.jwt_secret",
        value_type: ConfigValueType::String,
        default_fn: default_jwt_secret,
        requires_restart: true,
        is_sensitive: true,
        category: categories::AUTH,
        description: "JWT token signing secret key",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::API_TRUSTED_PROXIES,
        label_i18n_key: "config.keys.api.trusted_proxies",
        description_i18n_key: "config.descriptions.api.trusted_proxies",
        value_type: ConfigValueType::StringArray,
        default_fn: default_trusted_proxies,
        normalize_fn: Some(normalize_trusted_proxies),
        requires_restart: true,
        category: categories::AUTH,
        description: "Trusted proxy IPs or CIDRs (e.g., [\"10.0.0.1\", \"192.168.1.0/24\"]). Empty = trust no proxies, use connection IP only.",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::API_ACCESS_TOKEN_MINUTES,
        label_i18n_key: "config.keys.api.access_token_minutes",
        description_i18n_key: "config.descriptions.api.access_token_minutes",
        value_type: ConfigValueType::Number,
        default_fn: default_access_token_minutes,
        normalize_fn: Some(normalize_unsigned_integer),
        requires_restart: true,
        category: categories::AUTH,
        description: "Access token expiration time in minutes",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::API_REFRESH_TOKEN_DAYS,
        label_i18n_key: "config.keys.api.refresh_token_days",
        description_i18n_key: "config.descriptions.api.refresh_token_days",
        value_type: ConfigValueType::Number,
        default_fn: default_refresh_token_days,
        normalize_fn: Some(normalize_unsigned_integer),
        requires_restart: true,
        category: categories::AUTH,
        description: "Refresh token expiration time in days",
        ..ConfigDefinition::private_system()
    },
    // ========== Cookie 配置 (cookie) ==========
    ConfigDefinition {
        key: keys::API_COOKIE_SECURE,
        label_i18n_key: "config.keys.api.cookie_secure",
        description_i18n_key: "config.descriptions.api.cookie_secure",
        value_type: ConfigValueType::Boolean,
        default_fn: default_cookie_secure,
        category: categories::COOKIE,
        description: "Enable secure flag for cookies (HTTPS only)",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::API_COOKIE_SAME_SITE,
        label_i18n_key: "config.keys.api.cookie_same_site",
        description_i18n_key: "config.descriptions.api.cookie_same_site",
        value_type: ConfigValueType::StringEnum,
        default_fn: default_cookie_same_site,
        normalize_fn: Some(normalize_same_site),
        category: categories::COOKIE,
        description: "Cookie SameSite policy",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::API_COOKIE_DOMAIN,
        label_i18n_key: "config.keys.api.cookie_domain",
        description_i18n_key: "config.descriptions.api.cookie_domain",
        value_type: ConfigValueType::String,
        default_fn: default_empty,
        category: categories::COOKIE,
        description: "Cookie domain (empty for current domain)",
        ..ConfigDefinition::private_system()
    },
    // ========== 功能配置 (features) ==========
    ConfigDefinition {
        key: keys::FEATURES_RANDOM_CODE_LENGTH,
        label_i18n_key: "config.keys.features.random_code_length",
        description_i18n_key: "config.descriptions.features.random_code_length",
        value_type: ConfigValueType::Number,
        default_fn: default_random_code_length,
        normalize_fn: Some(normalize_unsigned_integer),
        category: categories::FEATURES,
        description: "Length of randomly generated short codes",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::FEATURES_DEFAULT_URL,
        label_i18n_key: "config.keys.features.default_url",
        description_i18n_key: "config.descriptions.features.default_url",
        value_type: ConfigValueType::String,
        default_fn: default_default_url,
        category: categories::FEATURES,
        description: "Default redirect URL for invalid short codes",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::FEATURES_ENABLE_ADMIN_PANEL,
        label_i18n_key: "config.keys.features.enable_admin_panel",
        description_i18n_key: "config.descriptions.features.enable_admin_panel",
        value_type: ConfigValueType::Boolean,
        default_fn: default_enable_admin_panel,
        requires_restart: true,
        category: categories::FEATURES,
        description: "Enable admin panel interface",
        ..ConfigDefinition::private_system()
    },
    // ========== 点击追踪 (tracking) ==========
    ConfigDefinition {
        key: keys::CLICK_ENABLE_TRACKING,
        label_i18n_key: "config.keys.click.enable_tracking",
        description_i18n_key: "config.descriptions.click.enable_tracking",
        value_type: ConfigValueType::Boolean,
        default_fn: default_enable_tracking,
        requires_restart: true,
        category: categories::TRACKING,
        description: "Enable click tracking and analytics",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::CLICK_FLUSH_INTERVAL,
        label_i18n_key: "config.keys.click.flush_interval",
        description_i18n_key: "config.descriptions.click.flush_interval",
        value_type: ConfigValueType::Number,
        default_fn: default_flush_interval,
        normalize_fn: Some(normalize_unsigned_integer),
        requires_restart: true,
        category: categories::TRACKING,
        description: "Click data flush interval in seconds",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::CLICK_MAX_CLICKS_BEFORE_FLUSH,
        label_i18n_key: "config.keys.click.max_clicks_before_flush",
        description_i18n_key: "config.descriptions.click.max_clicks_before_flush",
        value_type: ConfigValueType::Number,
        default_fn: default_max_clicks_before_flush,
        normalize_fn: Some(normalize_unsigned_integer),
        requires_restart: true,
        category: categories::TRACKING,
        description: "Maximum clicks before forcing flush",
        ..ConfigDefinition::private_system()
    },
    // ========== 路由配置 (routes) ==========
    ConfigDefinition {
        key: keys::ROUTES_ADMIN_PREFIX,
        label_i18n_key: "config.keys.routes.admin_prefix",
        description_i18n_key: "config.descriptions.routes.admin_prefix",
        value_type: ConfigValueType::String,
        default_fn: default_admin_prefix,
        requires_restart: true,
        category: categories::ROUTES,
        description: "Admin API route prefix",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ROUTES_HEALTH_PREFIX,
        label_i18n_key: "config.keys.routes.health_prefix",
        description_i18n_key: "config.descriptions.routes.health_prefix",
        value_type: ConfigValueType::String,
        default_fn: default_health_prefix,
        requires_restart: true,
        category: categories::ROUTES,
        description: "Health check route prefix",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ROUTES_FRONTEND_PREFIX,
        label_i18n_key: "config.keys.routes.frontend_prefix",
        description_i18n_key: "config.descriptions.routes.frontend_prefix",
        value_type: ConfigValueType::String,
        default_fn: default_frontend_prefix,
        requires_restart: true,
        category: categories::ROUTES,
        description: "Frontend assets route prefix",
        ..ConfigDefinition::private_system()
    },
    // ========== CORS 配置 (cors) ==========
    ConfigDefinition {
        key: keys::CORS_ENABLED,
        label_i18n_key: "config.keys.cors.enabled",
        description_i18n_key: "config.descriptions.cors.enabled",
        value_type: ConfigValueType::Boolean,
        default_fn: default_cors_enabled,
        requires_restart: true,
        category: categories::CORS,
        description: "Enable CORS configuration. When disabled, uses browser's same-origin policy (no cross-origin requests allowed)",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::CORS_ALLOWED_ORIGINS,
        label_i18n_key: "config.keys.cors.allowed_origins",
        description_i18n_key: "config.descriptions.cors.allowed_origins",
        value_type: ConfigValueType::StringArray,
        default_fn: default_cors_allowed_origins,
        requires_restart: true,
        category: categories::CORS,
        description: "Allowed origins for CORS (JSON array). Use [\"*\"] to allow any origin, empty array means same-origin only",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::CORS_ALLOWED_METHODS,
        label_i18n_key: "config.keys.cors.allowed_methods",
        description_i18n_key: "config.descriptions.cors.allowed_methods",
        value_type: ConfigValueType::StringEnumSet,
        default_fn: default_cors_allowed_methods,
        normalize_fn: Some(normalize_http_methods),
        requires_restart: true,
        category: categories::CORS,
        description: "Allowed HTTP methods for CORS",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::CORS_ALLOWED_HEADERS,
        label_i18n_key: "config.keys.cors.allowed_headers",
        description_i18n_key: "config.descriptions.cors.allowed_headers",
        value_type: ConfigValueType::StringArray,
        default_fn: default_cors_allowed_headers,
        requires_restart: true,
        category: categories::CORS,
        description: "Allowed headers for CORS (JSON array)",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::CORS_MAX_AGE,
        label_i18n_key: "config.keys.cors.max_age",
        description_i18n_key: "config.descriptions.cors.max_age",
        value_type: ConfigValueType::Number,
        default_fn: default_cors_max_age,
        normalize_fn: Some(normalize_unsigned_integer),
        requires_restart: true,
        category: categories::CORS,
        description: "CORS preflight cache duration in seconds",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::CORS_ALLOW_CREDENTIALS,
        label_i18n_key: "config.keys.cors.allow_credentials",
        description_i18n_key: "config.descriptions.cors.allow_credentials",
        value_type: ConfigValueType::Boolean,
        default_fn: default_cors_allow_credentials,
        requires_restart: true,
        category: categories::CORS,
        description: "Allow credentials in CORS requests. Cannot be used with wildcard origins for security reasons",
        ..ConfigDefinition::private_system()
    },
    // ========== 详细分析统计 (analytics) ==========
    ConfigDefinition {
        key: keys::ANALYTICS_ENABLE_DETAILED_LOGGING,
        label_i18n_key: "config.keys.analytics.enable_detailed_logging",
        description_i18n_key: "config.descriptions.analytics.enable_detailed_logging",
        value_type: ConfigValueType::Boolean,
        default_fn: default_analytics_enable_detailed_logging,
        requires_restart: true,
        category: categories::ANALYTICS,
        description: "Enable detailed click logging (writes to click_logs table)",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ANALYTICS_LOG_RETENTION_DAYS,
        label_i18n_key: "config.keys.analytics.log_retention_days",
        description_i18n_key: "config.descriptions.analytics.log_retention_days",
        value_type: ConfigValueType::Number,
        default_fn: default_analytics_log_retention_days,
        normalize_fn: Some(normalize_unsigned_integer),
        category: categories::ANALYTICS,
        description: "Raw click log retention period in days (cleaned by DataRetentionTask)",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ANALYTICS_ENABLE_IP_LOGGING,
        label_i18n_key: "config.keys.analytics.enable_ip_logging",
        description_i18n_key: "config.descriptions.analytics.enable_ip_logging",
        value_type: ConfigValueType::Boolean,
        default_fn: default_analytics_enable_ip_logging,
        category: categories::ANALYTICS,
        description: "Enable IP address logging (disable for privacy compliance)",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ANALYTICS_ENABLE_GEO_LOOKUP,
        label_i18n_key: "config.keys.analytics.enable_geo_lookup",
        description_i18n_key: "config.descriptions.analytics.enable_geo_lookup",
        value_type: ConfigValueType::Boolean,
        default_fn: default_analytics_enable_geo_lookup,
        category: categories::ANALYTICS,
        description: "Enable geographic location lookup for IP addresses",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ANALYTICS_HOURLY_RETENTION_DAYS,
        label_i18n_key: "config.keys.analytics.hourly_retention_days",
        description_i18n_key: "config.descriptions.analytics.hourly_retention_days",
        value_type: ConfigValueType::Number,
        default_fn: default_analytics_hourly_retention_days,
        normalize_fn: Some(normalize_unsigned_integer),
        category: categories::ANALYTICS,
        description: "Hourly rollup data retention period in days",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ANALYTICS_DAILY_RETENTION_DAYS,
        label_i18n_key: "config.keys.analytics.daily_retention_days",
        description_i18n_key: "config.descriptions.analytics.daily_retention_days",
        value_type: ConfigValueType::Number,
        default_fn: default_analytics_daily_retention_days,
        normalize_fn: Some(normalize_unsigned_integer),
        category: categories::ANALYTICS,
        description: "Daily rollup data retention period in days",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ANALYTICS_ENABLE_AUTO_ROLLUP,
        label_i18n_key: "config.keys.analytics.enable_auto_rollup",
        description_i18n_key: "config.descriptions.analytics.enable_auto_rollup",
        value_type: ConfigValueType::Boolean,
        default_fn: default_analytics_enable_auto_rollup,
        requires_restart: true,
        category: categories::ANALYTICS,
        description: "Enable automatic rollup aggregation and data cleanup",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ANALYTICS_SAMPLE_RATE,
        label_i18n_key: "config.keys.analytics.sample_rate",
        description_i18n_key: "config.descriptions.analytics.sample_rate",
        value_type: ConfigValueType::Number,
        default_fn: default_analytics_sample_rate,
        normalize_fn: Some(normalize_sample_rate),
        category: categories::ANALYTICS,
        description: "Click log sampling rate (0.0-1.0). 1.0 = log all clicks, 0.1 = log 10% of clicks",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ANALYTICS_MAX_LOG_ROWS,
        label_i18n_key: "config.keys.analytics.max_log_rows",
        description_i18n_key: "config.descriptions.analytics.max_log_rows",
        value_type: ConfigValueType::Number,
        default_fn: default_analytics_max_log_rows,
        normalize_fn: Some(normalize_unsigned_integer),
        category: categories::ANALYTICS,
        description: "Maximum rows in click_logs table. 0 = unlimited",
        ..ConfigDefinition::private_system()
    },
    ConfigDefinition {
        key: keys::ANALYTICS_MAX_ROWS_ACTION,
        label_i18n_key: "config.keys.analytics.max_rows_action",
        description_i18n_key: "config.descriptions.analytics.max_rows_action",
        value_type: ConfigValueType::StringEnum,
        default_fn: default_analytics_max_rows_action,
        normalize_fn: Some(normalize_max_rows_action),
        category: categories::ANALYTICS,
        description: "Action when max_log_rows exceeded: 'cleanup' (delete oldest) or 'stop' (stop logging)",
        ..ConfigDefinition::private_system()
    },
    // ========== UTM 追踪 (analytics) ==========
    ConfigDefinition {
        key: keys::UTM_ENABLE_PASSTHROUGH,
        label_i18n_key: "config.keys.utm.enable_passthrough",
        description_i18n_key: "config.descriptions.utm.enable_passthrough",
        value_type: ConfigValueType::Boolean,
        default_fn: default_utm_enable_passthrough,
        category: categories::ANALYTICS,
        description: "Enable UTM parameter passthrough to target URL (utm_source/medium/campaign/term/content)",
        ..ConfigDefinition::private_system()
    },
    // ========== 缓存配置 (cache) ==========
    ConfigDefinition {
        key: keys::CACHE_BLOOM_REBUILD_INTERVAL,
        label_i18n_key: "config.keys.cache.bloom_rebuild_interval",
        description_i18n_key: "config.descriptions.cache.bloom_rebuild_interval",
        value_type: ConfigValueType::Number,
        default_fn: default_bloom_rebuild_interval,
        normalize_fn: Some(normalize_unsigned_integer),
        requires_restart: true,
        category: categories::CACHE,
        description: "Bloom filter periodic rebuild interval in seconds (0 = disabled)",
        ..ConfigDefinition::private_system()
    },
];
}

/// Shortlinker 产品配置 action。
pub fn action_for_key(key: &str) -> Option<ActionType> {
    match key {
        keys::API_JWT_SECRET => Some(ActionType::GenerateToken),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_keys_and_categories_are_valid() {
        CONFIG_REGISTRY.validate_unique_keys().unwrap();
        CONFIG_REGISTRY
            .validate_categories(&[
                categories::AUTH,
                categories::COOKIE,
                categories::FEATURES,
                categories::ROUTES,
                categories::CORS,
                categories::TRACKING,
                categories::ANALYTICS,
                categories::CACHE,
            ])
            .unwrap();
    }

    #[test]
    fn registry_exposes_sensitive_metadata() {
        let def = CONFIG_REGISTRY.get(keys::API_ADMIN_TOKEN);
        assert!(def.is_some());
        let def = def.unwrap();
        assert_eq!(def.key, "api.admin_token");
        assert!(def.is_sensitive);
    }

    #[test]
    fn defaults_pass_registry_normalization() {
        let seeds = CONFIG_REGISTRY.default_seed_records().unwrap();
        assert_eq!(seeds.len(), CONFIG_REGISTRY.definitions().len());
        assert!(seeds.iter().any(|seed| seed.key == keys::API_ADMIN_TOKEN));
    }

    #[test]
    fn product_normalizers_canonicalize_values() {
        let lookup = std::collections::HashMap::new();
        assert_eq!(
            CONFIG_REGISTRY
                .normalize_value(&lookup, keys::API_COOKIE_SAME_SITE, "strict")
                .unwrap(),
            "Strict"
        );
        assert_eq!(
            CONFIG_REGISTRY
                .normalize_value(&lookup, keys::CORS_ALLOWED_METHODS, r#"["post","GET"]"#,)
                .unwrap(),
            r#"["GET","POST"]"#
        );
        assert!(
            CONFIG_REGISTRY
                .normalize_value(&lookup, keys::ANALYTICS_SAMPLE_RATE, "1.01")
                .is_err()
        );
        assert_eq!(
            CONFIG_REGISTRY
                .normalize_value(&lookup, keys::ANALYTICS_SAMPLE_RATE, " 1 ")
                .unwrap(),
            "1.0"
        );
        assert_eq!(
            CONFIG_REGISTRY
                .normalize_value(&lookup, keys::FEATURES_RANDOM_CODE_LENGTH, "006")
                .unwrap(),
            "6"
        );
    }
}
