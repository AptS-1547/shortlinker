use std::env;
use std::fs;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use arc_swap::ArcSwap;

use super::AppConfig;

static CONFIG: OnceLock<ArcSwap<AppConfig>> = OnceLock::new();

impl AppConfig {
    /// Load configuration from TOML file with environment variable fallback
    ///
    /// Uses default "config.toml" file (warns if doesn't exist)
    pub fn load() -> Self {
        let mut config = Self::load_from_file();
        #[allow(deprecated)]
        config.override_with_env();
        config
    }

    /// Load configuration from TOML file
    ///
    /// # Behavior
    /// - Uses "config.toml" in current directory
    /// - If file doesn't exist: silently uses in-memory defaults
    fn load_from_file() -> Self {
        let path = "config.toml";

        // Check if file exists - silently use defaults if not
        if !Path::new(path).exists() {
            return Self::default();
        }

        // Load the file
        match fs::read_to_string(path) {
            Ok(content) => match toml::from_str::<AppConfig>(&content) {
                Ok(config) => {
                    eprintln!("[INFO] Configuration loaded from: {}", path);
                    config
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to parse config file {}: {}", path, e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("[ERROR] Failed to read config file {}: {}", path, e);
                Self::default()
            }
        }
    }

    /// Override configuration with environment variables
    ///
    /// # Deprecated
    ///
    /// 此方法将在 0.5.0 版本中移除。环境变量配置覆盖逻辑已不再需要。
    #[deprecated(since = "0.4.0", note = "将在 0.5.0 版本中移除")]
    fn override_with_env(&mut self) {
        // Server config
        if let Ok(host) = env::var("SERVER_HOST") {
            self.server.host = host;
        }
        if let Ok(port) = env::var("SERVER_PORT") {
            if let Ok(port) = port.parse() {
                self.server.port = port;
            } else {
                eprintln!("[ERROR] Invalid SERVER_PORT: {}", port);
            }
        }
        if let Ok(socket) = env::var("UNIX_SOCKET") {
            self.server.unix_socket = Some(socket);
        }
        if let Ok(cpu_count) = env::var("CPU_COUNT") {
            if let Ok(count) = cpu_count.parse() {
                self.server.cpu_count = count;
            } else {
                eprintln!("[ERROR] Invalid CPU_COUNT: {}", cpu_count);
            }
        }

        // Database config
        if let Ok(database_url) = env::var("DATABASE_URL") {
            self.database.database_url = database_url;
        }
        if let Ok(pool_size) = env::var("DATABASE_POOL_SIZE") {
            if let Ok(size) = pool_size.parse::<u32>() {
                self.database.pool_size = size;
            } else {
                eprintln!("[ERROR] Invalid DATABASE_POLL_SIZE: {}", pool_size);
            }
        }
        if let Ok(timeout) = env::var("DATABASE_TIMEOUT") {
            if let Ok(t) = timeout.parse::<u64>() {
                self.database.timeout = t;
            } else {
                eprintln!("[ERROR] Invalid DATABASE_TIMEOUT: {}", timeout);
            }
        }

        // Cache config
        if let Ok(cache_type) = env::var("CACHE_TYPE") {
            self.cache.cache_type = cache_type;
        }
        if let Ok(default_ttl) = env::var("CACHE_DEFAULT_TTL") {
            if let Ok(ttl) = default_ttl.parse() {
                self.cache.default_ttl = ttl;
            } else {
                eprintln!("[ERROR] Invalid CACHE_DEFAULT_TTL: {}", default_ttl);
            }
        }
        if let Ok(redis_url) = env::var("REDIS_URL") {
            self.cache.redis.url = redis_url;
        }
        if let Ok(redis_key_prefix) = env::var("REDIS_KEY_PREFIX") {
            self.cache.redis.key_prefix = redis_key_prefix;
        }
        if let Ok(memory_max_capacity) = env::var("MEMORY_MAX_CAPACITY") {
            if let Ok(capacity) = memory_max_capacity.parse() {
                self.cache.memory.max_capacity = capacity;
            } else {
                eprintln!(
                    "[ERROR] Invalid MEMORY_MAX_CAPACITY: {}",
                    memory_max_capacity
                );
            }
        }

        // API config
        if let Ok(admin_token) = env::var("ADMIN_TOKEN") {
            self.api.admin_token = admin_token;
        }
        if let Ok(health_token) = env::var("HEALTH_TOKEN") {
            self.api.health_token = health_token;
        }

        // Route config
        if let Ok(admin_prefix) = env::var("ADMIN_ROUTE_PREFIX") {
            self.routes.admin_prefix = admin_prefix;
        }
        if let Ok(health_prefix) = env::var("HEALTH_ROUTE_PREFIX") {
            self.routes.health_prefix = health_prefix;
        }
        if let Ok(frontend_prefix) = env::var("FRONTEND_ROUTE_PREFIX") {
            self.routes.frontend_prefix = frontend_prefix;
        }

        // Feature config
        if let Ok(enable_admin_panel) = env::var("ENABLE_ADMIN_PANEL") {
            self.features.enable_admin_panel = enable_admin_panel == "true";
        }
        if let Ok(random_code_length) = env::var("RANDOM_CODE_LENGTH") {
            if let Ok(length) = random_code_length.parse() {
                self.features.random_code_length = length;
            } else {
                eprintln!("[ERROR] Invalid RANDOM_CODE_LENGTH: {}", random_code_length);
            }
        }
        if let Ok(default_url) = env::var("DEFAULT_URL") {
            self.features.default_url = default_url;
        }

        // Logging config
        if let Ok(log_level) = env::var("RUST_LOG") {
            self.logging.level = log_level;
        }
    }

    /// Generate a sample TOML configuration file
    pub fn generate_sample_config() -> String {
        let sample_config = AppConfig::default();
        toml::to_string_pretty(&sample_config)
            .unwrap_or_else(|e| format!("Error generating sample config: {}", e))
    }

    /// Save current configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;

        // Create parent directories if needed
        if let Some(parent) = path.as_ref().parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;
        }

        fs::write(path, content)?;
        Ok(())
    }
}

// Global configuration instance

/// Get the global configuration instance
///
/// Returns an Arc pointer to the configuration, which is cheap to clone
/// and doesn't hold any locks.
pub fn get_config() -> Arc<AppConfig> {
    CONFIG
        .get()
        .expect("Config not initialized. Call init_config() first.")
        .load_full()
}

/// Initialize the global configuration
///
/// Loads configuration from "config.toml" in the current directory.
/// If the file doesn't exist, uses in-memory defaults.
///
/// # Examples
/// ```no_run
/// use shortlinker::config::init_config;
/// init_config();
/// ```
pub fn init_config() {
    // Initialize the configuration
    CONFIG.get_or_init(|| ArcSwap::from_pointee(AppConfig::load()));
}

/// Update configuration with a closure (for runtime updates)
///
/// This performs a copy-on-write update: clones the current config,
/// applies the modification, then atomically replaces the old config.
///
/// # Arguments
/// * `updater` - A closure that modifies the AppConfig
///
/// # Example
/// ```no_run
/// use shortlinker::config::update_config;
/// update_config(|config| {
///     config.api.admin_token = "new_token".to_string();
/// });
/// ```
pub fn update_config<F>(updater: F)
where
    F: FnOnce(&mut AppConfig),
{
    let config = CONFIG
        .get()
        .expect("Config not initialized. Call init_config() first.");

    // Clone current config, modify it, then atomically replace
    let mut new_config = (*config.load_full()).clone();
    updater(&mut new_config);
    config.store(Arc::new(new_config));
}

/// Update a specific config field by key
///
/// Returns:
/// - `Ok(true)` if the key was found and updated successfully
/// - `Ok(false)` if the key was not found
/// - `Err(message)` if the key was found but the value was invalid
///
/// # 维护说明
///
/// 添加新配置时，需要在此函数的 match 中添加对应的映射。
/// 这是目前仍需手动维护的地方之一（另一处在 `config_migration.rs` 的 `get_config_value`）。
///
/// 如果遗漏了某个 key，编译器不会报错，配置更新会静默失败。
pub fn update_config_by_key(key: &str, value: &str) -> Result<bool, String> {
    use std::cell::Cell;

    let found = Cell::new(true);
    let error: Cell<Option<String>> = Cell::new(None);

    update_config(|config| {
        apply_config_key(config, key, value, &found, &error);
    });

    if let Some(err) = error.take() {
        return Err(err);
    }
    Ok(found.get())
}

/// 批量更新配置（单次 clone + 单次 store）
///
/// 相比逐个调用 `update_config_by_key`，这个函数只执行一次 clone 和一次 atomic store，
/// 避免了中间态不一致和多次内存分配。
///
/// # Returns
/// 返回更新失败的 key 及其错误信息列表。空列表表示全部成功。
pub fn batch_update_config_by_keys(
    updates: &std::collections::HashMap<String, String>,
) -> Vec<(String, String)> {
    use std::cell::Cell;

    let mut errors = Vec::new();

    update_config(|config| {
        for (key, value) in updates {
            let found = Cell::new(true);
            let error: Cell<Option<String>> = Cell::new(None);

            apply_config_key(config, key, value, &found, &error);

            if let Some(err) = error.take() {
                errors.push((key.clone(), err));
            } else if !found.get() {
                errors.push((key.clone(), format!("Unknown config key: {}", key)));
            }
        }
    });

    errors
}

/// 内部函数：应用单个 key 到 config（不触发 store）
fn apply_config_key(
    config: &mut AppConfig,
    key: &str,
    value: &str,
    found: &std::cell::Cell<bool>,
    error: &std::cell::Cell<Option<String>>,
) {
    use super::definitions::keys;

    match key {
        // API 认证配置
        keys::API_ADMIN_TOKEN => config.api.admin_token = value.to_string(),
        keys::API_HEALTH_TOKEN => config.api.health_token = value.to_string(),
        keys::API_JWT_SECRET => config.api.jwt_secret = value.to_string(),
        keys::API_ACCESS_TOKEN_MINUTES => match value.parse() {
            Ok(v) => config.api.access_token_minutes = v,
            Err(_) => error.set(Some(format!("Invalid number for {}: '{}'", key, value))),
        },
        keys::API_REFRESH_TOKEN_DAYS => match value.parse() {
            Ok(v) => config.api.refresh_token_days = v,
            Err(_) => error.set(Some(format!("Invalid number for {}: '{}'", key, value))),
        },

        // Cookie 配置
        keys::API_COOKIE_SECURE => {
            config.api.cookie_secure = value == "true" || value == "1" || value == "yes";
        }
        keys::API_COOKIE_SAME_SITE => match value.parse() {
            Ok(v) => config.api.cookie_same_site = v,
            Err(_) => error.set(Some(format!(
                "Invalid SameSite policy for {}: '{}'. Expected: strict, lax, none",
                key, value
            ))),
        },
        keys::API_COOKIE_DOMAIN => {
            config.api.cookie_domain = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
        }

        // 功能配置
        keys::FEATURES_RANDOM_CODE_LENGTH => match value.parse() {
            Ok(v) => config.features.random_code_length = v,
            Err(_) => error.set(Some(format!("Invalid number for {}: '{}'", key, value))),
        },
        keys::FEATURES_DEFAULT_URL => config.features.default_url = value.to_string(),
        keys::FEATURES_ENABLE_ADMIN_PANEL => {
            config.features.enable_admin_panel = value == "true" || value == "1" || value == "yes";
        }

        // 点击统计配置
        keys::CLICK_ENABLE_TRACKING => {
            config.click_manager.enable_click_tracking =
                value == "true" || value == "1" || value == "yes";
        }
        keys::CLICK_FLUSH_INTERVAL => match value.parse() {
            Ok(v) => config.click_manager.flush_interval = v,
            Err(_) => error.set(Some(format!("Invalid number for {}: '{}'", key, value))),
        },
        keys::CLICK_MAX_CLICKS_BEFORE_FLUSH => match value.parse() {
            Ok(v) => config.click_manager.max_clicks_before_flush = v,
            Err(_) => error.set(Some(format!("Invalid number for {}: '{}'", key, value))),
        },

        // 路由配置
        keys::ROUTES_ADMIN_PREFIX => config.routes.admin_prefix = value.to_string(),
        keys::ROUTES_HEALTH_PREFIX => config.routes.health_prefix = value.to_string(),
        keys::ROUTES_FRONTEND_PREFIX => config.routes.frontend_prefix = value.to_string(),

        // CORS 配置
        keys::CORS_ENABLED => {
            config.cors.enabled = value == "true" || value == "1" || value == "yes";
        }
        keys::CORS_ALLOWED_ORIGINS => match serde_json::from_str(value) {
            Ok(v) => config.cors.allowed_origins = v,
            Err(_) => error.set(Some(format!("Invalid JSON array for {}: '{}'", key, value))),
        },
        keys::CORS_ALLOWED_METHODS => match serde_json::from_str(value) {
            Ok(v) => config.cors.allowed_methods = v,
            Err(_) => error.set(Some(format!("Invalid JSON array for {}: '{}'", key, value))),
        },
        keys::CORS_ALLOWED_HEADERS => match serde_json::from_str(value) {
            Ok(v) => config.cors.allowed_headers = v,
            Err(_) => error.set(Some(format!("Invalid JSON array for {}: '{}'", key, value))),
        },
        keys::CORS_MAX_AGE => match value.parse() {
            Ok(v) => config.cors.max_age = v,
            Err(_) => error.set(Some(format!("Invalid number for {}: '{}'", key, value))),
        },
        keys::CORS_ALLOW_CREDENTIALS => {
            config.cors.allow_credentials = value == "true" || value == "1" || value == "yes";
        }

        _ => found.set(false),
    }
}
