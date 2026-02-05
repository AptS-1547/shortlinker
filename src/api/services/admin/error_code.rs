//! 统一 API 错误码定义

use serde_repr::{Deserialize_repr, Serialize_repr};
use ts_rs::TS;

use super::types::TS_EXPORT_PATH;

/// API 错误码枚举
///
/// 使用 serde_repr 序列化为数字，ts-rs 自动生成 TypeScript 类型。
/// 按千位分域：
/// - 0: 成功
/// - 1000-1099: 通用错误
/// - 2000-2099: 认证错误
/// - 3000-3099: 链接错误
/// - 4000-4099: 导入导出错误
/// - 5000-5099: 配置错误
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
#[ts(rename = "ErrorCode")]
#[ts(repr(enum))]
#[repr(i32)]
pub enum ErrorCode {
    // 成功
    Success = 0,

    // 通用错误 1000-1099
    BadRequest = 1000,
    Unauthorized = 1001,
    NotFound = 1004,
    InternalServerError = 1005,
    BatchSizeTooLarge = 1010,
    FileTooLarge = 1011,
    InvalidDateFormat = 1012,
    ServiceUnavailable = 1030,

    // 认证错误 2000-2099
    AuthFailed = 2000,
    TokenExpired = 2001,
    TokenInvalid = 2002,
    CsrfInvalid = 2003,
    RateLimitExceeded = 2004,

    // 链接错误 3000-3099
    LinkNotFound = 3000,
    LinkAlreadyExists = 3001,
    LinkInvalidUrl = 3002,
    LinkInvalidExpireTime = 3003,
    LinkPasswordHashError = 3004,
    LinkDatabaseError = 3005,
    LinkEmptyCode = 3006,

    // 导入导出错误 4000-4099
    ImportFailed = 4000,
    ExportFailed = 4001,
    InvalidMultipartData = 4002,
    FileReadError = 4003,
    CsvFileMissing = 4004,
    CsvParseError = 4005,
    CsvGenerationError = 4006,

    // 配置错误 5000-5099
    ConfigNotFound = 5000,
    ConfigUpdateFailed = 5001,
    ConfigReloadFailed = 5002,

    // Analytics 错误 6000-6099
    AnalyticsQueryFailed = 6000,
    AnalyticsLinkNotFound = 6001,
    AnalyticsInvalidDateRange = 6002,
}
