use serde::{Deserialize, Serialize};

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub routes: RouteConfig,
    #[serde(default)]
    pub features: FeatureConfig,
    #[serde(default)]
    pub click_manager: ClickManagerConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_host")]
    pub host: String,
    #[serde(default = "default_server_port")]
    pub port: u16,
    #[serde(default)]
    pub unix_socket: Option<String>,
    #[serde(default = "default_cpu_count")]
    pub cpu_count: usize,
}

/// 数据库连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_database_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_database_timeout")]
    pub timeout: u64,
}

/// 缓存系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(rename = "type")]
    #[serde(default = "default_cache_type")]
    pub cache_type: String,
    #[serde(default = "default_cache_ttl")]
    pub default_ttl: u64,
    #[serde(default)]
    pub redis: RedisConfig,
    #[serde(default)]
    pub memory: MemoryConfig,
}

/// Redis 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    #[serde(default = "default_redis_url")]
    pub url: String,
    #[serde(default = "default_redis_key_prefix")]
    pub key_prefix: String,
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: u64,
}

/// 内存缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_memory_capacity")]
    pub max_capacity: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiConfig {
    #[serde(default)]
    pub admin_token: String,
    #[serde(default)]
    pub health_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    #[serde(default = "default_admin_prefix")]
    pub admin_prefix: String,
    #[serde(default = "default_health_prefix")]
    pub health_prefix: String,
    #[serde(default = "default_frontend_prefix")]
    pub frontend_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureConfig {
    #[serde(default)]
    pub enable_admin_panel: bool,
    #[serde(default = "default_random_code_length")]
    pub random_code_length: usize,
    #[serde(default = "default_default_url")]
    pub default_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClickManagerConfig {
    #[serde(default = "default_enable_click_tracking")]
    pub enable_click_tracking: bool,
    #[serde(default = "default_flush_interval")]
    pub flush_interval: u64,
    #[serde(default = "default_max_clicks_before_flush")]
    pub max_clicks_before_flush: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default = "default_log_file")]
    pub file: Option<String>,
    #[serde(default = "default_max_size")]
    pub max_size: u64,
    #[serde(default = "default_max_backups")]
    pub max_backups: u32,
    #[serde(default = "default_enable_rotation")]
    pub enable_rotation: bool,
}

// Default value functions
fn default_server_host() -> String {
    "127.0.0.1".to_string()
}

fn default_server_port() -> u16 {
    8080
}

fn default_cpu_count() -> usize {
    num_cpus::get()
}

fn default_database_backend() -> String {
    "sqlite".to_string()
}

fn default_database_url() -> String {
    "shortlinks.db".to_string()
}

fn default_database_pool_size() -> u32 {
    10
}

fn default_database_timeout() -> u64 {
    30
}

fn default_cache_type() -> String {
    "memory".to_string()
}

fn default_cache_ttl() -> u64 {
    3600
}

fn default_redis_url() -> String {
    "redis://127.0.0.1:6379/".to_string()
}

fn default_redis_key_prefix() -> String {
    "shortlinker:".to_string()
}

fn default_redis_pool_size() -> u64 {
    10
}

fn default_memory_capacity() -> u64 {
    10000
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

fn default_random_code_length() -> usize {
    6
}

fn default_default_url() -> String {
    "https://esap.cc/repo".to_string()
}

fn default_enable_click_tracking() -> bool {
    true
}

fn default_flush_interval() -> u64 {
    30
}

fn default_max_clicks_before_flush() -> u64 {
    100
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "text".to_string()
}

fn default_log_file() -> Option<String> {
    None
}

fn default_max_size() -> u64 {
    100
}

fn default_max_backups() -> u32 {
    5
}

fn default_enable_rotation() -> bool {
    true
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_server_host(),
            port: default_server_port(),
            unix_socket: None,
            cpu_count: default_cpu_count(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: default_database_url(),
            pool_size: default_database_pool_size(),
            timeout: default_database_timeout(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            cache_type: default_cache_type(),
            default_ttl: default_cache_ttl(),
            redis: RedisConfig::default(),
            memory: MemoryConfig::default(),
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: default_redis_url(),
            key_prefix: default_redis_key_prefix(),
            pool_size: default_redis_pool_size(),
        }
    }
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_capacity: default_memory_capacity(),
        }
    }
}

impl Default for RouteConfig {
    fn default() -> Self {
        Self {
            admin_prefix: default_admin_prefix(),
            health_prefix: default_health_prefix(),
            frontend_prefix: default_frontend_prefix(),
        }
    }
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            enable_admin_panel: false,
            random_code_length: default_random_code_length(),
            default_url: default_default_url(),
        }
    }
}

impl Default for ClickManagerConfig {
    fn default() -> Self {
        Self {
            enable_click_tracking: default_enable_click_tracking(),
            flush_interval: default_flush_interval(),
            max_clicks_before_flush: default_max_clicks_before_flush(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            file: default_log_file(),
            max_size: default_max_size(),
            max_backups: default_max_backups(),
            enable_rotation: default_enable_rotation(),
        }
    }
}
