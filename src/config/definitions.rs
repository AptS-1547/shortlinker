//! 配置定义模块 - 单一数据源
//!
//! 所有配置项的元信息都在这里定义，包括：
//! - key 字符串
//! - 值类型
//! - 默认值函数
//! - 元信息（requires_restart, is_sensitive, editable, category, description）
//!
//! 其他模块（schema, runtime_config）都从这里读取配置定义。
//!
//! # 添加新配置的步骤
//!
//! 1. 在 `keys` 模块中添加新的 key 常量
//! 2. 添加默认值函数（如果需要动态默认值）
//! 3. 在 `ALL_CONFIGS` 数组中添加 `ConfigDef` 定义

use super::types::{RustType, ValueType};

/// 配置分类常量
pub mod categories {
    pub const AUTH: &str = "auth";
    pub const COOKIE: &str = "cookie";
    pub const FEATURES: &str = "features";
    pub const ROUTES: &str = "routes";
    pub const CORS: &str = "cors";
    pub const TRACKING: &str = "tracking";
    pub const ANALYTICS: &str = "analytics";
}

/// 配置项完整定义
pub struct ConfigDef {
    /// 配置键，如 "api.admin_token"
    pub key: &'static str,
    /// 数据库/前端值类型
    pub value_type: ValueType,
    /// Rust 代码中的类型
    pub rust_type: RustType,
    /// 默认值生成函数
    pub default_fn: fn() -> String,
    /// 是否需要重启生效
    pub requires_restart: bool,
    /// 是否敏感（如密码、token）
    pub is_sensitive: bool,
    /// 是否可在前端编辑
    pub editable: bool,
    /// 配置分类
    pub category: &'static str,
    /// 描述（英文）
    pub description: &'static str,
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
}

// 默认值函数
fn default_empty() -> String {
    String::new()
}

fn default_admin_token() -> String {
    crate::utils::generate_secure_token(32) // 使用加密安全的 OsRng
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

/// 所有配置定义（单一数据源）
pub static ALL_CONFIGS: &[ConfigDef] = &[
    // ========== API 认证 (auth) ==========
    ConfigDef {
        key: keys::API_ADMIN_TOKEN,
        value_type: ValueType::String,
        rust_type: RustType::String,
        default_fn: default_admin_token,
        requires_restart: false,
        is_sensitive: true,
        editable: true,
        category: categories::AUTH,
        description: "Admin API authentication token (Argon2 hashed)",
    },
    ConfigDef {
        key: keys::API_HEALTH_TOKEN,
        value_type: ValueType::String,
        rust_type: RustType::String,
        default_fn: default_empty,
        requires_restart: false,
        is_sensitive: true,
        editable: true,
        category: categories::AUTH,
        description: "Health check endpoint authentication token",
    },
    ConfigDef {
        key: keys::API_JWT_SECRET,
        value_type: ValueType::String,
        rust_type: RustType::String,
        default_fn: default_jwt_secret,
        requires_restart: false,
        is_sensitive: true,
        editable: true,
        category: categories::AUTH,
        description: "JWT token signing secret key",
    },
    ConfigDef {
        key: keys::API_TRUSTED_PROXIES,
        value_type: ValueType::StringArray,
        rust_type: RustType::VecString,
        default_fn: default_trusted_proxies,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::AUTH,
        description: "Trusted proxy IPs or CIDRs (e.g., [\"10.0.0.1\", \"192.168.1.0/24\"]). Empty = trust no proxies, use connection IP only.",
    },
    ConfigDef {
        key: keys::API_ACCESS_TOKEN_MINUTES,
        value_type: ValueType::Int,
        rust_type: RustType::U64,
        default_fn: default_access_token_minutes,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::AUTH,
        description: "Access token expiration time in minutes",
    },
    ConfigDef {
        key: keys::API_REFRESH_TOKEN_DAYS,
        value_type: ValueType::Int,
        rust_type: RustType::U64,
        default_fn: default_refresh_token_days,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::AUTH,
        description: "Refresh token expiration time in days",
    },
    // ========== Cookie 配置 (cookie) ==========
    ConfigDef {
        key: keys::API_COOKIE_SECURE,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_cookie_secure,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::COOKIE,
        description: "Enable secure flag for cookies (HTTPS only)",
    },
    ConfigDef {
        key: keys::API_COOKIE_SAME_SITE,
        value_type: ValueType::Enum,
        rust_type: RustType::SameSitePolicy,
        default_fn: default_cookie_same_site,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::COOKIE,
        description: "Cookie SameSite policy",
    },
    ConfigDef {
        key: keys::API_COOKIE_DOMAIN,
        value_type: ValueType::String,
        rust_type: RustType::OptionString,
        default_fn: default_empty,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::COOKIE,
        description: "Cookie domain (empty for current domain)",
    },
    // ========== 功能配置 (features) ==========
    ConfigDef {
        key: keys::FEATURES_RANDOM_CODE_LENGTH,
        value_type: ValueType::Int,
        rust_type: RustType::Usize,
        default_fn: default_random_code_length,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::FEATURES,
        description: "Length of randomly generated short codes",
    },
    ConfigDef {
        key: keys::FEATURES_DEFAULT_URL,
        value_type: ValueType::String,
        rust_type: RustType::String,
        default_fn: default_default_url,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::FEATURES,
        description: "Default redirect URL for invalid short codes",
    },
    ConfigDef {
        key: keys::FEATURES_ENABLE_ADMIN_PANEL,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_enable_admin_panel,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::FEATURES,
        description: "Enable admin panel interface",
    },
    // ========== 点击追踪 (tracking) ==========
    ConfigDef {
        key: keys::CLICK_ENABLE_TRACKING,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_enable_tracking,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::TRACKING,
        description: "Enable click tracking and analytics",
    },
    ConfigDef {
        key: keys::CLICK_FLUSH_INTERVAL,
        value_type: ValueType::Int,
        rust_type: RustType::U64,
        default_fn: default_flush_interval,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::TRACKING,
        description: "Click data flush interval in seconds",
    },
    ConfigDef {
        key: keys::CLICK_MAX_CLICKS_BEFORE_FLUSH,
        value_type: ValueType::Int,
        rust_type: RustType::U64,
        default_fn: default_max_clicks_before_flush,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::TRACKING,
        description: "Maximum clicks before forcing flush",
    },
    // ========== 路由配置 (routes) ==========
    ConfigDef {
        key: keys::ROUTES_ADMIN_PREFIX,
        value_type: ValueType::String,
        rust_type: RustType::String,
        default_fn: default_admin_prefix,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::ROUTES,
        description: "Admin API route prefix",
    },
    ConfigDef {
        key: keys::ROUTES_HEALTH_PREFIX,
        value_type: ValueType::String,
        rust_type: RustType::String,
        default_fn: default_health_prefix,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::ROUTES,
        description: "Health check route prefix",
    },
    ConfigDef {
        key: keys::ROUTES_FRONTEND_PREFIX,
        value_type: ValueType::String,
        rust_type: RustType::String,
        default_fn: default_frontend_prefix,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::ROUTES,
        description: "Frontend assets route prefix",
    },
    // ========== CORS 配置 (cors) ==========
    ConfigDef {
        key: keys::CORS_ENABLED,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_cors_enabled,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::CORS,
        description: "Enable CORS configuration. When disabled, uses browser's same-origin policy (no cross-origin requests allowed)",
    },
    ConfigDef {
        key: keys::CORS_ALLOWED_ORIGINS,
        value_type: ValueType::StringArray,
        rust_type: RustType::VecString,
        default_fn: default_cors_allowed_origins,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::CORS,
        description: "Allowed origins for CORS (JSON array). Use [\"*\"] to allow any origin, empty array means same-origin only",
    },
    ConfigDef {
        key: keys::CORS_ALLOWED_METHODS,
        value_type: ValueType::EnumArray,
        rust_type: RustType::VecHttpMethod,
        default_fn: default_cors_allowed_methods,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::CORS,
        description: "Allowed HTTP methods for CORS",
    },
    ConfigDef {
        key: keys::CORS_ALLOWED_HEADERS,
        value_type: ValueType::StringArray,
        rust_type: RustType::VecString,
        default_fn: default_cors_allowed_headers,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::CORS,
        description: "Allowed headers for CORS (JSON array)",
    },
    ConfigDef {
        key: keys::CORS_MAX_AGE,
        value_type: ValueType::Int,
        rust_type: RustType::U64,
        default_fn: default_cors_max_age,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::CORS,
        description: "CORS preflight cache duration in seconds",
    },
    ConfigDef {
        key: keys::CORS_ALLOW_CREDENTIALS,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_cors_allow_credentials,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::CORS,
        description: "Allow credentials in CORS requests. Cannot be used with wildcard origins for security reasons",
    },
    // ========== 详细分析统计 (analytics) ==========
    ConfigDef {
        key: keys::ANALYTICS_ENABLE_DETAILED_LOGGING,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_analytics_enable_detailed_logging,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Enable detailed click logging (writes to click_logs table)",
    },
    ConfigDef {
        key: keys::ANALYTICS_LOG_RETENTION_DAYS,
        value_type: ValueType::Int,
        rust_type: RustType::U64,
        default_fn: default_analytics_log_retention_days,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Raw click log retention period in days (cleaned by DataRetentionTask)",
    },
    ConfigDef {
        key: keys::ANALYTICS_ENABLE_IP_LOGGING,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_analytics_enable_ip_logging,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Enable IP address logging (disable for privacy compliance)",
    },
    ConfigDef {
        key: keys::ANALYTICS_ENABLE_GEO_LOOKUP,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_analytics_enable_geo_lookup,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Enable geographic location lookup for IP addresses",
    },
    ConfigDef {
        key: keys::ANALYTICS_HOURLY_RETENTION_DAYS,
        value_type: ValueType::Int,
        rust_type: RustType::U64,
        default_fn: default_analytics_hourly_retention_days,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Hourly rollup data retention period in days",
    },
    ConfigDef {
        key: keys::ANALYTICS_DAILY_RETENTION_DAYS,
        value_type: ValueType::Int,
        rust_type: RustType::U64,
        default_fn: default_analytics_daily_retention_days,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Daily rollup data retention period in days",
    },
    ConfigDef {
        key: keys::ANALYTICS_ENABLE_AUTO_ROLLUP,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_analytics_enable_auto_rollup,
        requires_restart: true,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Enable automatic rollup aggregation and data cleanup",
    },
    ConfigDef {
        key: keys::ANALYTICS_SAMPLE_RATE,
        value_type: ValueType::Float,
        rust_type: RustType::F64,
        default_fn: default_analytics_sample_rate,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Click log sampling rate (0.0-1.0). 1.0 = log all clicks, 0.1 = log 10% of clicks",
    },
    ConfigDef {
        key: keys::ANALYTICS_MAX_LOG_ROWS,
        value_type: ValueType::Int,
        rust_type: RustType::U64,
        default_fn: default_analytics_max_log_rows,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Maximum rows in click_logs table. 0 = unlimited",
    },
    ConfigDef {
        key: keys::ANALYTICS_MAX_ROWS_ACTION,
        value_type: ValueType::Enum,
        rust_type: RustType::MaxRowsAction,
        default_fn: default_analytics_max_rows_action,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Action when max_log_rows exceeded: 'cleanup' (delete oldest) or 'stop' (stop logging)",
    },
    // ========== UTM 追踪 (analytics) ==========
    ConfigDef {
        key: keys::UTM_ENABLE_PASSTHROUGH,
        value_type: ValueType::Bool,
        rust_type: RustType::Bool,
        default_fn: default_utm_enable_passthrough,
        requires_restart: false,
        is_sensitive: false,
        editable: true,
        category: categories::ANALYTICS,
        description: "Enable UTM parameter passthrough to target URL (utm_source/medium/campaign/term/content)",
    },
];

/// 根据 key 查找配置定义
pub fn get_def(key: &str) -> Option<&'static ConfigDef> {
    ALL_CONFIGS.iter().find(|def| def.key == key)
}

/// 获取所有 key 列表
pub fn all_keys() -> impl Iterator<Item = &'static str> {
    ALL_CONFIGS.iter().map(|def| def.key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_configs_keys_unique() {
        let mut keys: Vec<_> = ALL_CONFIGS.iter().map(|d| d.key).collect();
        let original_len = keys.len();
        keys.sort();
        keys.dedup();
        assert_eq!(
            keys.len(),
            original_len,
            "Duplicate keys found in ALL_CONFIGS"
        );
    }

    #[test]
    fn test_get_def() {
        let def = get_def(keys::API_ADMIN_TOKEN);
        assert!(def.is_some());
        let def = def.unwrap();
        assert_eq!(def.key, "api.admin_token");
        assert!(def.is_sensitive);
    }

    #[test]
    fn test_all_keys() {
        let keys: Vec<_> = all_keys().collect();
        assert!(!keys.is_empty());
        assert!(keys.contains(&"api.admin_token"));
    }
}
