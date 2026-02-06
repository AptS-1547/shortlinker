use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, EnumMessage, IntoEnumIterator};
use ts_rs::TS;

/// 输出目录常量（用于测试导出）
#[cfg(test)]
use super::types::TS_EXPORT_PATH;

/// 获取默认的 HTTP 方法 JSON 数组
///
/// 使用 EnumIter 自动从 HttpMethod 枚举生成，保证类型安全。
/// 用于配置迁移和 schema 默认值。
pub fn default_http_methods_json() -> String {
    let methods: Vec<String> = HttpMethod::iter().map(|v| v.as_ref().to_string()).collect();
    serde_json::to_string(&methods).unwrap_or_else(|_| "[]".to_string())
}

/// Cookie SameSite 策略
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Default,
    TS,
    EnumIter,
    AsRefStr,
    EnumMessage,
)]
#[ts(export, export_to = "../admin-panel/src/services/types.generated.ts")]
#[serde(rename_all = "PascalCase")]
#[strum(serialize_all = "PascalCase")]
pub enum SameSitePolicy {
    #[strum(message = "Most secure, only same-site requests carry cookies")]
    Strict,
    #[default]
    #[strum(message = "Default, allows top-level navigation to carry cookies")]
    Lax,
    #[strum(message = "No restrictions, requires Secure attribute")]
    None,
}

impl std::fmt::Display for SameSitePolicy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Strict => write!(f, "Strict"),
            Self::Lax => write!(f, "Lax"),
            Self::None => write!(f, "None"),
        }
    }
}

impl std::str::FromStr for SameSitePolicy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "strict" => Ok(Self::Strict),
            "lax" => Ok(Self::Lax),
            "none" => Ok(Self::None),
            _ => Err(format!(
                "Invalid SameSite policy: '{}'. Valid: Strict, Lax, None",
                s
            )),
        }
    }
}

/// HTTP 方法枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, EnumIter, AsRefStr)]
#[ts(export, export_to = "../admin-panel/src/services/types.generated.ts")]
#[serde(rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Get => write!(f, "GET"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Patch => write!(f, "PATCH"),
            Self::Head => write!(f, "HEAD"),
            Self::Options => write!(f, "OPTIONS"),
        }
    }
}

impl std::str::FromStr for HttpMethod {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "PATCH" => Ok(Self::Patch),
            "HEAD" => Ok(Self::Head),
            "OPTIONS" => Ok(Self::Options),
            _ => Err(format!(
                "Invalid HTTP method: '{}'. Valid: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS",
                s
            )),
        }
    }
}

/// 静态配置（从 TOML 加载，启动时使用）
///
/// 包含基础设施配置：
/// - server: 服务器地址、端口、CPU 数量
/// - database: 数据库连接配置
/// - cache: 缓存系统配置
/// - logging: 日志配置
/// - analytics: 分析统计配置
/// - ipc: IPC 服务器配置
///
/// 运行时配置（api, routes, features, click_manager, cors）存储在数据库中，
/// 通过 Admin Panel 或 API 进行管理，使用 RuntimeConfig 读取。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaticConfig {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub analytics: AnalyticsConfig,
    #[serde(default)]
    pub ipc: IpcConfig,
}

impl StaticConfig {
    /// 从 TOML 文件和环境变量加载配置
    ///
    /// 优先级：ENV > config.toml > 默认值
    /// ENV 前缀：SL，分隔符：__
    /// 示例：SL__SERVER__PORT=9999
    pub fn load() -> Self {
        use config::{Config, Environment, File};

        let path = "config.toml";

        let builder = Config::builder()
            // 1. 从 TOML 文件加载（可选）
            .add_source(File::with_name(path).required(false))
            // 2. 从环境变量覆盖，前缀 SL，分隔符 __
            .add_source(
                Environment::with_prefix("SL")
                    .separator("__")
                    .try_parsing(true),
            );

        match builder.build() {
            Ok(settings) => match settings.try_deserialize::<StaticConfig>() {
                Ok(config) => {
                    if std::path::Path::new(path).exists() {
                        eprintln!("[INFO] Configuration loaded from: {}", path);
                    }
                    config
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to deserialize config: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("[ERROR] Failed to build config: {}", e);
                Self::default()
            }
        }
    }

    /// 生成示例 TOML 配置文件
    pub fn generate_sample_config() -> String {
        let sample_config = Self::default();
        toml::to_string_pretty(&sample_config)
            .unwrap_or_else(|e| format!("Error generating sample config: {}", e))
    }

    /// 保存配置到 TOML 文件
    pub fn save_to_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;

        // Create parent directories if needed
        if let Some(parent) = path.as_ref().parent()
            && !parent.exists()
        {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        Ok(())
    }
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
    #[serde(default = "default_retry_count")]
    pub retry_count: u32,
    #[serde(default = "default_retry_base_delay_ms")]
    pub retry_base_delay_ms: u64,
    #[serde(default = "default_retry_max_delay_ms")]
    pub retry_max_delay_ms: u64,
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
}

/// 内存缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    #[serde(default = "default_memory_capacity")]
    pub max_capacity: u64,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default = "default_log_file")]
    pub file: Option<String>,
    /// 单个日志文件最大大小（MB）
    ///
    /// **注意**: 当前版本未使用此字段，日志轮转按天而非按大小。
    /// 保留字段以保持配置文件向后兼容。
    #[serde(default = "default_max_size")]
    pub max_size: u64,
    #[serde(default = "default_max_backups")]
    pub max_backups: u32,
    #[serde(default = "default_enable_rotation")]
    pub enable_rotation: bool,
}

// ============================================================
// Default value functions for static config
// ============================================================

fn default_server_host() -> String {
    "127.0.0.1".to_string()
}

fn default_server_port() -> u16 {
    8080
}

fn default_cpu_count() -> usize {
    num_cpus::get()
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

fn default_retry_count() -> u32 {
    3
}

fn default_retry_base_delay_ms() -> u64 {
    100
}

fn default_retry_max_delay_ms() -> u64 {
    2000
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

fn default_memory_capacity() -> u64 {
    10000
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

// ============================================================
// Default implementations
// ============================================================

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
            retry_count: default_retry_count(),
            retry_base_delay_ms: default_retry_base_delay_ms(),
            retry_max_delay_ms: default_retry_max_delay_ms(),
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

/// 分析统计配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    /// MaxMindDB 文件路径 (GeoLite2-City.mmdb)
    /// 如果配置且文件可读，使用本地解析；否则 fallback 到外部 API
    #[serde(default)]
    pub maxminddb_path: Option<String>,

    /// 外部 GeoIP API URL (fallback)
    /// 使用 {ip} 作为占位符，例如: http://ip-api.com/json/{ip}?fields=status,countryCode,city
    #[serde(default = "default_geoip_api_url")]
    pub geoip_api_url: String,
}

fn default_geoip_api_url() -> String {
    "http://ip-api.com/json/{ip}?fields=status,countryCode,city".to_string()
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            maxminddb_path: None,
            geoip_api_url: default_geoip_api_url(),
        }
    }
}

/// IPC (进程间通信) 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcConfig {
    /// 是否启用 IPC 服务器
    /// 禁用后 CLI 的 status、reload 等命令将无法与服务器通信
    #[serde(default = "default_ipc_enabled")]
    pub enabled: bool,

    /// Socket 路径（Unix）或命名管道路径（Windows）
    /// Unix 默认: "./shortlinker.sock"
    /// Windows 默认: r"\\.\pipe\shortlinker"
    #[serde(default)]
    pub socket_path: Option<String>,

    /// 最大消息大小（字节）
    #[serde(default = "default_ipc_max_message_size")]
    pub max_message_size: usize,

    /// 默认超时（秒）
    #[serde(default = "default_ipc_timeout")]
    pub timeout: u64,

    /// Reload 操作超时（秒）
    #[serde(default = "default_ipc_reload_timeout")]
    pub reload_timeout: u64,

    /// 批量操作（导入/导出）超时（秒）
    #[serde(default = "default_ipc_bulk_timeout")]
    pub bulk_timeout: u64,
}

impl IpcConfig {
    /// 获取实际的 socket 路径
    ///
    /// 优先级: CLI --socket 参数 > config.toml > 平台默认值
    pub fn effective_socket_path(&self) -> String {
        // 1. CLI 参数覆盖
        if let Some(override_path) = crate::config::get_ipc_socket_override() {
            return override_path.clone();
        }

        // 2. config.toml 配置
        if let Some(ref path) = self.socket_path {
            return path.clone();
        }

        // 3. 平台默认值
        #[cfg(unix)]
        {
            "./shortlinker.sock".to_string()
        }
        #[cfg(windows)]
        {
            r"\\.\pipe\shortlinker".to_string()
        }
    }

    /// 获取默认超时 Duration
    pub fn default_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout)
    }

    /// 获取 reload 超时 Duration
    pub fn reload_timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.reload_timeout)
    }

    /// 获取批量操作超时 Duration
    pub fn bulk_timeout_duration(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.bulk_timeout)
    }
}

fn default_ipc_enabled() -> bool {
    true
}
fn default_ipc_max_message_size() -> usize {
    64 * 1024
}
fn default_ipc_timeout() -> u64 {
    5
}
fn default_ipc_reload_timeout() -> u64 {
    30
}
fn default_ipc_bulk_timeout() -> u64 {
    60
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            enabled: default_ipc_enabled(),
            socket_path: None,
            max_message_size: default_ipc_max_message_size(),
            timeout: default_ipc_timeout(),
            reload_timeout: default_ipc_reload_timeout(),
            bulk_timeout: default_ipc_bulk_timeout(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_typescript_types() {
        // 运行此测试会自动生成 TypeScript 类型文件
        // cargo test export_typescript_types -- --nocapture
        let cfg = ts_rs::Config::default();

        // Export enums
        SameSitePolicy::export_all(&cfg).expect("Failed to export SameSitePolicy");
        HttpMethod::export_all(&cfg).expect("Failed to export HttpMethod");

        println!("TypeScript types exported to {}", TS_EXPORT_PATH);
    }
}
