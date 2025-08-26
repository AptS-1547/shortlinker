use std::env;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub storage: StorageConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default)]
    pub api: ApiConfig,
    #[serde(default)]
    pub routes: RouteConfig,
    #[serde(default)]
    pub features: FeatureConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_storage_backend")]
    pub backend: String,
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_db_file_name")]
    pub db_file_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    #[serde(default = "default_redis_url")]
    pub redis_url: String,
    #[serde(default = "default_redis_key_prefix")]
    pub redis_key_prefix: String,
    #[serde(default = "default_redis_ttl")]
    pub redis_ttl: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
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

fn default_storage_backend() -> String {
    "sqlite".to_string()
}

fn default_database_url() -> String {
    "shortlinks.db".to_string()
}

fn default_db_file_name() -> String {
    "links.json".to_string()
}

fn default_redis_url() -> String {
    "redis://127.0.0.1:6379/".to_string()
}

fn default_redis_key_prefix() -> String {
    "shortlinker:".to_string()
}

fn default_redis_ttl() -> u64 {
    3600
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

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            storage: StorageConfig::default(),
            cache: CacheConfig::default(),
            api: ApiConfig::default(),
            routes: RouteConfig::default(),
            features: FeatureConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
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

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: default_storage_backend(),
            database_url: default_database_url(),
            db_file_name: default_db_file_name(),
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: default_redis_url(),
            redis_key_prefix: default_redis_key_prefix(),
            redis_ttl: default_redis_ttl(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            admin_token: String::new(),
            health_token: String::new(),
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

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

impl Config {
    /// Load configuration from TOML file with environment variable fallback
    pub fn load() -> Self {
        let mut config = Self::load_from_file();
        config.override_with_env();
        config
    }

    /// Load configuration from TOML file
    fn load_from_file() -> Self {
        let config_paths = [
            "config.toml",
            "shortlinker.toml",
            "config/config.toml",
            "/etc/shortlinker/config.toml",
        ];

        for path in &config_paths {
            if Path::new(path).exists() {
                debug!("Loading config from: {}", path);
                match fs::read_to_string(path) {
                    Ok(content) => {
                        match toml::from_str::<Config>(&content) {
                            Ok(config) => {
                                debug!("Successfully loaded config from: {}", path);
                                return config;
                            }
                            Err(e) => {
                                warn!("Failed to parse config file {}: {}", path, e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read config file {}: {}", path, e);
                    }
                }
            }
        }

        debug!("No config file found, using defaults");
        Self::default()
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
            }
        }
        if let Ok(socket) = env::var("UNIX_SOCKET") {
            self.server.unix_socket = Some(socket);
        }
        if let Ok(cpu_count) = env::var("CPU_COUNT") {
            if let Ok(count) = cpu_count.parse() {
                self.server.cpu_count = count;
            }
        }

        // Storage config
        if let Ok(backend) = env::var("STORAGE_BACKEND") {
            self.storage.backend = backend;
        }
        if let Ok(database_url) = env::var("DATABASE_URL") {
            self.storage.database_url = database_url;
        }
        if let Ok(db_file_name) = env::var("DB_FILE_NAME") {
            self.storage.db_file_name = db_file_name;
        }

        // Cache config
        if let Ok(redis_url) = env::var("REDIS_URL") {
            self.cache.redis_url = redis_url;
        }
        if let Ok(redis_key_prefix) = env::var("REDIS_KEY_PREFIX") {
            self.cache.redis_key_prefix = redis_key_prefix;
        }
        if let Ok(redis_ttl) = env::var("REDIS_TTL") {
            if let Ok(ttl) = redis_ttl.parse() {
                self.cache.redis_ttl = ttl;
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
        let sample_config = Config::default();
        toml::to_string_pretty(&sample_config).unwrap_or_else(|e| {
            format!("Error generating sample config: {}", e)
        })
    }

    /// Save current configuration to a TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

// Global configuration instance
use std::sync::OnceLock;
static CONFIG: OnceLock<Config> = OnceLock::new();

/// Get the global configuration instance
pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::load)
}

/// Initialize the global configuration
pub fn init_config() {
    CONFIG.get_or_init(Config::load);
}
