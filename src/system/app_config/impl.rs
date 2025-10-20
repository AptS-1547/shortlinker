use std::env;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

use super::AppConfig;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();
static CONFIG_PATH: OnceLock<String> = OnceLock::new();

impl AppConfig {
    /// Load configuration from TOML file with environment variable fallback
    ///
    /// # Arguments
    /// * `config_path` - Optional path to configuration file
    ///   - `Some(path)`: Use specified file (create if doesn't exist)
    ///   - `None`: Use default "config.toml" (warn if doesn't exist)
    pub fn load(config_path: Option<&str>) -> Self {
        let mut config = Self::load_from_file(config_path);
        config.override_with_env();
        config
    }

    /// Load configuration from TOML file
    ///
    /// # Behavior
    /// - If `config_path` is provided and file doesn't exist: create default config file
    /// - If `config_path` is None (default) and file doesn't exist: warn and use in-memory defaults
    fn load_from_file(config_path: Option<&str>) -> Self {
        let path = config_path.unwrap_or("config.toml");
        let is_custom_path = config_path.is_some();

        // Check if file exists
        if !Path::new(path).exists() {
            if is_custom_path {
                // User specified a custom path: create the file
                eprintln!("[WARN] Configuration file not found: {}", path);
                eprintln!("[WARN] Creating default configuration file...");
                if let Err(e) = Self::ensure_config_file(path) {
                    eprintln!("[ERROR] Failed to create config file {}: {}", path, e);
                    eprintln!("[WARN] Using in-memory default configuration");
                    return Self::default();
                }
                eprintln!("[INFO] Configuration file created: {}", path);
            } else {
                // Default path: just warn
                eprintln!("[WARN] No configuration file found at: {}", path);
                eprintln!("[WARN] Using in-memory default configuration");
                eprintln!("[HINT] Use -c/--config to specify a custom configuration file");
                return Self::default();
            }
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
                    eprintln!("[WARN] Using in-memory default configuration");
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("[ERROR] Failed to read config file {}: {}", path, e);
                eprintln!("[WARN] Using in-memory default configuration");
                Self::default()
            }
        }
    }

    /// Ensure configuration file exists, create with defaults if not
    fn ensure_config_file(path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let default_config = Self::default();
        let content = toml::to_string_pretty(&default_config)?;

        // Create parent directories if needed
        if let Some(parent) = Path::new(path).parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(path, content)?;
        Ok(())
    }

    /// Override configuration with environment variables
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
        if let Ok(backend) = env::var("DATABASE_BACKEND") {
            self.database.backend = backend;
        }
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
        if let Ok(redis_pool_size) = env::var("REDIS_POOL_SIZE") {
            if let Ok(size) = redis_pool_size.parse() {
                self.cache.redis.pool_size = size;
            } else {
                eprintln!("[ERROR] Invalid REDIS_POOL_SIZE: {}", redis_pool_size);
            }
        }
        if let Ok(memory_max_capacity) = env::var("MEMORY_MAX_CAPACITY") {
            if let Ok(capacity) = memory_max_capacity.parse() {
                self.cache.memory.max_capacity = capacity;
            } else {
                eprintln!("[ERROR] Invalid MEMORY_MAX_CAPACITY: {}", memory_max_capacity);
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
        if let Some(parent) = path.as_ref().parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        fs::write(path, content)?;
        Ok(())
    }
}

// Global configuration instance

/// Get the global configuration instance
pub fn get_config() -> &'static AppConfig {
    CONFIG
        .get()
        .expect("Config not initialized. Call init_config() first.")
}

/// Initialize the global configuration
///
/// # Arguments
/// * `config_path` - Optional path to configuration file
///   - `Some(path)`: Load from specified file (create if doesn't exist)
///   - `None`: Load from default "config.toml" (warn if doesn't exist)
///
/// # Examples
/// ```
/// // Use default config.toml
/// init_config(None);
///
/// // Use custom configuration file
/// init_config(Some("custom.toml".to_string()));
/// ```
pub fn init_config(config_path: Option<String>) {
    // Store the config path for potential later use
    if let Some(path) = &config_path {
        CONFIG_PATH.set(path.clone()).ok();
    }

    // Initialize the configuration
    CONFIG.get_or_init(|| AppConfig::load(config_path.as_deref()));
}

/// Get the configuration file path that was used
pub fn get_config_path() -> Option<&'static str> {
    CONFIG_PATH.get().map(|s| s.as_str())
}
