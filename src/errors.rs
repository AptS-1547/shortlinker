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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ShortlinkerError::cache_connection("test").code(), "E001");
        assert_eq!(
            ShortlinkerError::cache_plugin_not_found("test").code(),
            "E002"
        );
        assert_eq!(ShortlinkerError::database_config("test").code(), "E003");
        assert_eq!(ShortlinkerError::database_connection("test").code(), "E004");
        assert_eq!(ShortlinkerError::database_operation("test").code(), "E005");
        assert_eq!(ShortlinkerError::file_operation("test").code(), "E006");
        assert_eq!(ShortlinkerError::validation("test").code(), "E007");
        assert_eq!(ShortlinkerError::not_found("test").code(), "E008");
        assert_eq!(ShortlinkerError::serialization("test").code(), "E009");
        assert_eq!(ShortlinkerError::signal_operation("test").code(), "E010");
        assert_eq!(
            ShortlinkerError::storage_plugin_not_found("test").code(),
            "E011"
        );
        assert_eq!(ShortlinkerError::date_parse("test").code(), "E012");
        assert_eq!(ShortlinkerError::notify_server("test").code(), "E013");
    }

    #[test]
    fn test_error_types() {
        assert_eq!(
            ShortlinkerError::cache_connection("test").error_type(),
            "Cache Connection Error"
        );
        assert_eq!(
            ShortlinkerError::database_operation("test").error_type(),
            "Database Operation Error"
        );
        assert_eq!(
            ShortlinkerError::not_found("test").error_type(),
            "Resource Not Found"
        );
        assert_eq!(
            ShortlinkerError::validation("test").error_type(),
            "Validation Error"
        );
    }

    #[test]
    fn test_error_message() {
        let err = ShortlinkerError::validation("Invalid input");
        assert_eq!(err.message(), "Invalid input");

        let err = ShortlinkerError::not_found("Link not found");
        assert_eq!(err.message(), "Link not found");
    }

    #[test]
    fn test_format_simple() {
        let err = ShortlinkerError::validation("Invalid URL");
        let formatted = err.format_simple();
        assert!(formatted.contains("Validation Error"));
        assert!(formatted.contains("Invalid URL"));
    }

    #[test]
    fn test_display_trait() {
        let err = ShortlinkerError::not_found("Resource missing");
        let display = format!("{}", err);
        assert!(display.contains("Resource Not Found"));
        assert!(display.contains("Resource missing"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ShortlinkerError = io_err.into();
        assert_eq!(err.code(), "E006");
        assert!(err.message().contains("file not found"));
    }

    #[test]
    fn test_from_serde_json_error() {
        let json_err = serde_json::from_str::<String>("invalid").unwrap_err();
        let err: ShortlinkerError = json_err.into();
        assert_eq!(err.code(), "E009");
    }

    #[test]
    fn test_from_chrono_parse_error() {
        let chrono_err = "invalid"
            .parse::<chrono::DateTime<chrono::Utc>>()
            .unwrap_err();
        let err: ShortlinkerError = chrono_err.into();
        assert_eq!(err.code(), "E012");
    }

    #[test]
    fn test_convenience_constructors() {
        // 测试 Into<String> 泛型参数
        let err1 = ShortlinkerError::validation("test");
        let err2 = ShortlinkerError::validation(String::from("test"));
        assert_eq!(err1.message(), err2.message());
    }

    #[test]
    fn test_error_is_clone() {
        let err = ShortlinkerError::validation("test");
        let cloned = err.clone();
        assert_eq!(err.message(), cloned.message());
        assert_eq!(err.code(), cloned.code());
    }

    #[test]
    fn test_error_is_debug() {
        let err = ShortlinkerError::validation("test");
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Validation"));
    }
}
