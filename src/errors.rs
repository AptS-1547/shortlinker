use std::fmt;

/// 定义错误类型的宏
///
/// 自动生成：
/// - enum 定义
/// - code() 方法
/// - error_type() 方法
/// - message() 方法
macro_rules! define_shortlinker_errors {
    ($(
        $variant:ident($code:literal, $type_name:literal)
    ),* $(,)?) => {
        #[derive(Debug, Clone)]
        pub enum ShortlinkerError {
            $($variant(String),)*
        }

        impl ShortlinkerError {
            /// 获取错误代码
            pub fn code(&self) -> &'static str {
                match self {
                    $(ShortlinkerError::$variant(_) => $code,)*
                }
            }

            /// 获取错误类型名称
            pub fn error_type(&self) -> &'static str {
                match self {
                    $(ShortlinkerError::$variant(_) => $type_name,)*
                }
            }

            /// 获取错误详情
            pub fn message(&self) -> &str {
                match self {
                    $(ShortlinkerError::$variant(msg) => msg,)*
                }
            }
        }
    };
}

define_shortlinker_errors! {
    CacheConnection("E001", "Cache Connection Error"),
    CachePluginNotFound("E002", "Cache Plugin Not Found"),
    DatabaseConfig("E003", "Database Configuration Error"),
    DatabaseConnection("E004", "Database Connection Error"),
    DatabaseOperation("E005", "Database Operation Error"),
    FileOperation("E006", "File Operation Error"),
    Validation("E007", "Validation Error"),
    NotFound("E008", "Resource Not Found"),
    Serialization("E009", "Serialization Error"),
    SignalOperation("E010", "Signal Operation Error"),
    StoragePluginNotFound("E011", "Storage Plugin Not Found"),
    DateParse("E012", "Date Parse Error"),
    NotifyServer("E013", "Notify Server Error"),
}

impl ShortlinkerError {
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
