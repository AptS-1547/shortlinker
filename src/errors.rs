use std::fmt;

#[derive(Debug, Clone)]
pub enum ShortlinkerError {
    CacheConnection(String),
    CachePluginNotFound(String),
    DatabaseConfig(String),
    DatabaseConnection(String),
    DatabaseOperation(String),
    FileOperation(String),
    Validation(String),
    NotFound(String),
    Serialization(String),
    SignalOperation(String),
    StoragePluginNotFound(String),
    DateParse(String),
    NotifyServer(String),
}

impl ShortlinkerError {
    /// 获取错误代码
    pub fn code(&self) -> &'static str {
        match self {
            ShortlinkerError::CacheConnection(_) => "E001",
            ShortlinkerError::CachePluginNotFound(_) => "E002",
            ShortlinkerError::DatabaseConfig(_) => "E003",
            ShortlinkerError::DatabaseConnection(_) => "E004",
            ShortlinkerError::DatabaseOperation(_) => "E005",
            ShortlinkerError::FileOperation(_) => "E006",
            ShortlinkerError::Validation(_) => "E007",
            ShortlinkerError::NotFound(_) => "E008",
            ShortlinkerError::Serialization(_) => "E009",
            ShortlinkerError::SignalOperation(_) => "E010",
            ShortlinkerError::StoragePluginNotFound(_) => "E011",
            ShortlinkerError::DateParse(_) => "E012",
            ShortlinkerError::NotifyServer(_) => "E013",
        }
    }

    /// 获取错误类型名称
    pub fn error_type(&self) -> &'static str {
        match self {
            ShortlinkerError::CacheConnection(_) => "Cache Connection Error",
            ShortlinkerError::CachePluginNotFound(_) => "Cache Plugin Not Found",
            ShortlinkerError::DatabaseConfig(_) => "Database Configuration Error",
            ShortlinkerError::DatabaseConnection(_) => "Database Connection Error",
            ShortlinkerError::DatabaseOperation(_) => "Database Operation Error",
            ShortlinkerError::FileOperation(_) => "File Operation Error",
            ShortlinkerError::Validation(_) => "Validation Error",
            ShortlinkerError::NotFound(_) => "Resource Not Found",
            ShortlinkerError::Serialization(_) => "Serialization Error",
            ShortlinkerError::SignalOperation(_) => "Signal Operation Error",
            ShortlinkerError::StoragePluginNotFound(_) => "Storage Plugin Not Found",
            ShortlinkerError::DateParse(_) => "Date Parse Error",
            ShortlinkerError::NotifyServer(_) => "Notify Server Error",
        }
    }

    /// 获取错误详情
    pub fn message(&self) -> &str {
        match self {
            ShortlinkerError::CacheConnection(msg) => msg,
            ShortlinkerError::CachePluginNotFound(msg) => msg,
            ShortlinkerError::DatabaseConfig(msg) => msg,
            ShortlinkerError::DatabaseConnection(msg) => msg,
            ShortlinkerError::DatabaseOperation(msg) => msg,
            ShortlinkerError::FileOperation(msg) => msg,
            ShortlinkerError::Validation(msg) => msg,
            ShortlinkerError::NotFound(msg) => msg,
            ShortlinkerError::Serialization(msg) => msg,
            ShortlinkerError::SignalOperation(msg) => msg,
            ShortlinkerError::StoragePluginNotFound(msg) => msg,
            ShortlinkerError::DateParse(msg) => msg,
            ShortlinkerError::NotifyServer(msg) => msg,
        }
    }

    /// 格式化为彩色输出（用于 Server 模式）
    #[cfg(feature = "server")]
    pub fn format_colored(&self) -> String {
        use colored::Colorize;
        format!(
            "{} {} {}\n  {}",
            "[ERROR]".red().bold(),
            self.code().yellow(),
            self.error_type().red(),
            self.message().white()
        )
    }

    /// 格式化为简洁输出（用于 CLI/TUI 模式）
    pub fn format_simple(&self) -> String {
        format!("{}: {}", self.error_type(), self.message())
    }
}

impl fmt::Display for ShortlinkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // 默认使用简洁格式
        write!(f, "{}", self.format_simple())
    }
}

impl std::error::Error for ShortlinkerError {}

// 便捷的构造函数
impl ShortlinkerError {
    pub fn cache_connection<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::CacheConnection(msg.into())
    }

    pub fn cache_plugin_not_found<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::CachePluginNotFound(msg.into())
    }

    pub fn database_config<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::DatabaseConfig(msg.into())
    }

    pub fn database_connection<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::DatabaseConnection(msg.into())
    }

    pub fn database_operation<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::DatabaseOperation(msg.into())
    }

    pub fn file_operation<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::FileOperation(msg.into())
    }

    pub fn validation<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::Validation(msg.into())
    }

    pub fn not_found<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::NotFound(msg.into())
    }

    pub fn serialization<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::Serialization(msg.into())
    }

    pub fn signal_operation<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::SignalOperation(msg.into())
    }

    pub fn storage_plugin_not_found<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::StoragePluginNotFound(msg.into())
    }

    pub fn date_parse<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::DateParse(msg.into())
    }

    pub fn notify_server<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::NotifyServer(msg.into())
    }
}

// 为常见的错误类型实现 From trait
impl From<sea_orm::DbErr> for ShortlinkerError {
    fn from(err: sea_orm::DbErr) -> Self {
        ShortlinkerError::DatabaseOperation(err.to_string())
    }
}

impl From<std::io::Error> for ShortlinkerError {
    fn from(err: std::io::Error) -> Self {
        ShortlinkerError::FileOperation(err.to_string())
    }
}

impl From<serde_json::Error> for ShortlinkerError {
    fn from(err: serde_json::Error) -> Self {
        ShortlinkerError::Serialization(err.to_string())
    }
}

impl From<chrono::ParseError> for ShortlinkerError {
    fn from(err: chrono::ParseError) -> Self {
        ShortlinkerError::DateParse(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ShortlinkerError>;
