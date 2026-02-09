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
    // ========== E001-E010: 基础设施错误 ==========
    CacheConnection("E001", "Cache Connection Error"),
    CachePluginNotFound("E002", "Cache Plugin Not Found"),
    DatabaseConfig("E003", "Database Configuration Error"),
    DatabaseConnection("E004", "Database Connection Error"),
    DatabaseOperation("E005", "Database Operation Error"),
    FileOperation("E006", "File Operation Error"),
    Validation("E007", "Validation Error"),
    NotFound("E008", "Resource Not Found"),
    Serialization("E009", "Serialization Error"),
    NotifyServer("E010", "Notify Server Error"),

    // ========== E011-E019: 认证错误 ==========
    AuthPasswordInvalid("E011", "Password Invalid"),
    AuthTokenExpired("E012", "Token Expired"),
    AuthTokenInvalid("E013", "Token Invalid"),
    AuthRateLimitExceeded("E014", "Rate Limit Exceeded"),

    // ========== E020-E029: 链接业务错误 ==========
    LinkInvalidUrl("E020", "Invalid URL"),
    LinkAlreadyExists("E021", "Link Already Exists"),
    LinkInvalidExpireTime("E022", "Invalid Expire Time"),
    LinkPasswordHashError("E023", "Password Hash Error"),
    LinkInvalidCode("E024", "Invalid Short Code"),
    LinkReservedCode("E025", "Reserved Short Code"),

    // ========== E030-E039: 导入导出错误（保留，未来实现） ==========
    CsvParseFailed("E030", "CSV Parse Error"),
    CsvGenerationFailed("E031", "CSV Generation Error"),
    CsvFileMissing("E032", "CSV File Missing"),
    ImportFailed("E033", "Import Failed"),
    ExportFailed("E034", "Export Failed"),
    InvalidMultipartData("E035", "Invalid Multipart Data"),
    FileReadError("E036", "File Read Error"),

    // ========== E040-E049: 配置错误 ==========
    ConfigNotFound("E040", "Config Not Found"),
    ConfigUpdateFailed("E041", "Config Update Failed"),
    ConfigReloadFailed("E042", "Config Reload Failed"),

    // ========== E050-E059: 通用 HTTP 错误 ==========
    ServiceUnavailable("E050", "Service Unavailable"),
    InternalError("E051", "Internal Server Error"),

    // ========== E060-E069: Analytics 错误 ==========
    AnalyticsQueryFailed("E060", "Analytics Query Failed"),
    AnalyticsLinkNotFound("E061", "Analytics Link Not Found"),
    AnalyticsInvalidDateRange("E062", "Analytics Invalid Date Range"),
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

    /// 获取对应的 HTTP 状态码
    #[cfg(feature = "server")]
    pub fn http_status(&self) -> actix_web::http::StatusCode {
        use actix_web::http::StatusCode;
        match self {
            // 400 Bad Request
            Self::Validation(_)
            | Self::LinkInvalidUrl(_)
            | Self::LinkInvalidExpireTime(_)
            | Self::LinkInvalidCode(_)
            | Self::LinkReservedCode(_)
            | Self::InvalidMultipartData(_)
            | Self::CsvFileMissing(_)
            | Self::CsvParseFailed(_)
            | Self::AnalyticsInvalidDateRange(_) => StatusCode::BAD_REQUEST,

            // 401 Unauthorized
            Self::AuthPasswordInvalid(_)
            | Self::AuthTokenExpired(_)
            | Self::AuthTokenInvalid(_) => StatusCode::UNAUTHORIZED,

            // 404 Not Found
            Self::NotFound(_) | Self::ConfigNotFound(_) | Self::AnalyticsLinkNotFound(_) => {
                StatusCode::NOT_FOUND
            }

            // 409 Conflict
            Self::LinkAlreadyExists(_) => StatusCode::CONFLICT,

            // 429 Too Many Requests
            Self::AuthRateLimitExceeded(_) => StatusCode::TOO_MANY_REQUESTS,

            // 503 Service Unavailable
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,

            // 500 Internal Server Error (default)
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
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
    // 基础设施错误
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

    pub fn notify_server<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::NotifyServer(msg.into())
    }

    // 认证错误
    pub fn auth_password_invalid<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::AuthPasswordInvalid(msg.into())
    }

    pub fn auth_token_expired<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::AuthTokenExpired(msg.into())
    }

    pub fn auth_token_invalid<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::AuthTokenInvalid(msg.into())
    }

    pub fn auth_rate_limit_exceeded<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::AuthRateLimitExceeded(msg.into())
    }

    // 链接业务错误
    pub fn link_invalid_url<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::LinkInvalidUrl(msg.into())
    }

    pub fn link_already_exists<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::LinkAlreadyExists(msg.into())
    }

    pub fn link_invalid_expire_time<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::LinkInvalidExpireTime(msg.into())
    }

    pub fn link_password_hash_error<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::LinkPasswordHashError(msg.into())
    }

    pub fn link_invalid_code<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::LinkInvalidCode(msg.into())
    }

    pub fn link_reserved_code<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::LinkReservedCode(msg.into())
    }

    // 导入导出错误
    pub fn csv_parse_failed<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::CsvParseFailed(msg.into())
    }

    pub fn csv_generation_failed<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::CsvGenerationFailed(msg.into())
    }

    pub fn csv_file_missing<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::CsvFileMissing(msg.into())
    }

    pub fn import_failed<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::ImportFailed(msg.into())
    }

    pub fn export_failed<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::ExportFailed(msg.into())
    }

    pub fn invalid_multipart_data<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::InvalidMultipartData(msg.into())
    }

    pub fn file_read_error<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::FileReadError(msg.into())
    }

    // 配置错误
    pub fn config_not_found<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::ConfigNotFound(msg.into())
    }

    pub fn config_update_failed<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::ConfigUpdateFailed(msg.into())
    }

    pub fn config_reload_failed<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::ConfigReloadFailed(msg.into())
    }

    // 通用 HTTP 错误
    pub fn service_unavailable<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::ServiceUnavailable(msg.into())
    }

    pub fn internal_error<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::InternalError(msg.into())
    }

    // Analytics 错误
    pub fn analytics_query_failed<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::AnalyticsQueryFailed(msg.into())
    }

    pub fn analytics_link_not_found<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::AnalyticsLinkNotFound(msg.into())
    }

    pub fn analytics_invalid_date_range<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::AnalyticsInvalidDateRange(msg.into())
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

pub type Result<T> = std::result::Result<T, ShortlinkerError>;

/// ShortlinkerError → ErrorCode 自动转换
///
/// 仅在 server feature 下可用（ErrorCode 定义在 api 模块中）
#[cfg(feature = "server")]
impl From<ShortlinkerError> for crate::api::services::admin::error_code::ErrorCode {
    fn from(err: ShortlinkerError) -> Self {
        use crate::api::services::admin::error_code::ErrorCode;
        match err {
            // 认证错误
            ShortlinkerError::AuthPasswordInvalid(_) => ErrorCode::AuthFailed,
            ShortlinkerError::AuthTokenExpired(_) => ErrorCode::TokenExpired,
            ShortlinkerError::AuthTokenInvalid(_) => ErrorCode::TokenInvalid,
            ShortlinkerError::AuthRateLimitExceeded(_) => ErrorCode::RateLimitExceeded,

            // 链接错误
            ShortlinkerError::LinkInvalidUrl(_) => ErrorCode::LinkInvalidUrl,
            ShortlinkerError::LinkAlreadyExists(_) => ErrorCode::LinkAlreadyExists,
            ShortlinkerError::LinkInvalidExpireTime(_) => ErrorCode::LinkInvalidExpireTime,
            ShortlinkerError::LinkPasswordHashError(_) => ErrorCode::LinkPasswordHashError,
            ShortlinkerError::LinkInvalidCode(_) => ErrorCode::LinkInvalidCode,
            ShortlinkerError::LinkReservedCode(_) => ErrorCode::LinkReservedCode,

            // 导入导出错误
            ShortlinkerError::CsvParseFailed(_) => ErrorCode::CsvParseError,
            ShortlinkerError::CsvGenerationFailed(_) => ErrorCode::CsvGenerationError,
            ShortlinkerError::CsvFileMissing(_) => ErrorCode::CsvFileMissing,
            ShortlinkerError::ImportFailed(_) => ErrorCode::ImportFailed,
            ShortlinkerError::ExportFailed(_) => ErrorCode::ExportFailed,
            ShortlinkerError::InvalidMultipartData(_) => ErrorCode::InvalidMultipartData,
            ShortlinkerError::FileReadError(_) => ErrorCode::FileReadError,

            // 配置错误
            ShortlinkerError::ConfigNotFound(_) => ErrorCode::ConfigNotFound,
            ShortlinkerError::ConfigUpdateFailed(_) => ErrorCode::ConfigUpdateFailed,
            ShortlinkerError::ConfigReloadFailed(_) => ErrorCode::ConfigReloadFailed,

            // 通用错误
            ShortlinkerError::Validation(_) => ErrorCode::BadRequest,
            ShortlinkerError::ServiceUnavailable(_) => ErrorCode::ServiceUnavailable,
            ShortlinkerError::NotFound(_) => ErrorCode::NotFound,

            // Analytics 错误
            ShortlinkerError::AnalyticsQueryFailed(_) => ErrorCode::AnalyticsQueryFailed,
            ShortlinkerError::AnalyticsLinkNotFound(_) => ErrorCode::AnalyticsLinkNotFound,
            ShortlinkerError::AnalyticsInvalidDateRange(_) => ErrorCode::AnalyticsInvalidDateRange,

            // 其他基础设施错误 → InternalServerError
            _ => ErrorCode::InternalServerError,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        // 基础设施错误 E001-E010
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
        assert_eq!(ShortlinkerError::notify_server("test").code(), "E010");

        // 认证错误 E011-E014
        assert_eq!(
            ShortlinkerError::auth_password_invalid("test").code(),
            "E011"
        );
        assert_eq!(ShortlinkerError::auth_token_expired("test").code(), "E012");
        assert_eq!(ShortlinkerError::auth_token_invalid("test").code(), "E013");
        assert_eq!(
            ShortlinkerError::auth_rate_limit_exceeded("test").code(),
            "E014"
        );

        // 链接业务错误 E020-E023
        assert_eq!(ShortlinkerError::link_invalid_url("test").code(), "E020");
        assert_eq!(ShortlinkerError::link_already_exists("test").code(), "E021");
        assert_eq!(
            ShortlinkerError::link_invalid_expire_time("test").code(),
            "E022"
        );
        assert_eq!(
            ShortlinkerError::link_password_hash_error("test").code(),
            "E023"
        );

        // 导入导出错误 E030-E036
        assert_eq!(ShortlinkerError::csv_parse_failed("test").code(), "E030");
        assert_eq!(
            ShortlinkerError::csv_generation_failed("test").code(),
            "E031"
        );
        assert_eq!(ShortlinkerError::csv_file_missing("test").code(), "E032");
        assert_eq!(ShortlinkerError::import_failed("test").code(), "E033");
        assert_eq!(ShortlinkerError::export_failed("test").code(), "E034");
        assert_eq!(
            ShortlinkerError::invalid_multipart_data("test").code(),
            "E035"
        );
        assert_eq!(ShortlinkerError::file_read_error("test").code(), "E036");

        // 配置错误 E040-E042
        assert_eq!(ShortlinkerError::config_not_found("test").code(), "E040");
        assert_eq!(
            ShortlinkerError::config_update_failed("test").code(),
            "E041"
        );
        assert_eq!(
            ShortlinkerError::config_reload_failed("test").code(),
            "E042"
        );

        // 通用 HTTP 错误 E050-E051
        assert_eq!(ShortlinkerError::service_unavailable("test").code(), "E050");
        assert_eq!(ShortlinkerError::internal_error("test").code(), "E051");
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
        assert_eq!(
            ShortlinkerError::auth_password_invalid("test").error_type(),
            "Password Invalid"
        );
        assert_eq!(
            ShortlinkerError::link_invalid_url("test").error_type(),
            "Invalid URL"
        );
    }

    #[test]
    fn test_error_message() {
        let err = ShortlinkerError::validation("Invalid input");
        assert_eq!(err.message(), "Invalid input");

        let err = ShortlinkerError::not_found("Link not found");
        assert_eq!(err.message(), "Link not found");

        let err = ShortlinkerError::link_invalid_url("bad url");
        assert_eq!(err.message(), "bad url");
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
