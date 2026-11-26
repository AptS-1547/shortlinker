//! 统一 API 错误码定义

use serde_repr::{Deserialize_repr, Serialize_repr};

/// API 错误码枚举
///
/// 使用 serde_repr 序列化为数字，OpenAPI schema 保持相同 wire value。
/// 按千位分域：
/// - 0: 成功
/// - 1000-1099: 通用错误
/// - 2000-2099: 认证错误
/// - 3000-3099: 链接错误
/// - 4000-4099: 导入导出错误
/// - 5000-5099: 配置错误
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(strum::EnumIter))]
#[repr(i32)]
pub enum ErrorCode {
    // 成功
    /// 请求成功，业务逻辑正常完成。
    Success = 0,

    // 通用错误 1000-1099
    /// 请求参数非法或语义不合法。
    BadRequest = 1000,
    /// 未认证或缺少有效凭据。
    Unauthorized = 1001,
    /// 请求的资源不存在。
    NotFound = 1004,
    /// 未归类的服务器内部错误。
    InternalServerError = 1005,
    /// 批量操作条数超过单次请求上限。
    BatchSizeTooLarge = 1010,
    /// 上传文件体积超过大小限制。
    FileTooLarge = 1011,
    /// 日期参数格式非法。
    InvalidDateFormat = 1012,
    /// 依赖组件不可用，服务暂时无法处理请求。
    ServiceUnavailable = 1030,

    // 认证错误 2000-2099
    /// 认证失败，密码或凭据不正确。
    AuthFailed = 2000,
    /// 访问令牌已过期，需要刷新。
    TokenExpired = 2001,
    /// 令牌无效：签名、格式或声明非法。
    TokenInvalid = 2002,
    /// CSRF 校验失败或令牌缺失。
    CsrfInvalid = 2003,
    /// 认证尝试触发登录/刷新限流。
    RateLimitExceeded = 2004,

    // 链接错误 3000-3099
    /// 短链不存在。
    LinkNotFound = 3000,
    /// 短码已被其他链接占用。
    LinkAlreadyExists = 3001,
    /// 目标 URL 未通过格式校验。
    LinkInvalidUrl = 3002,
    /// 过期时间格式非法或早于当前时间。
    LinkInvalidExpireTime = 3003,
    /// 密码哈希计算失败。
    LinkPasswordHashError = 3004,
    /// 短链数据库读写失败。
    LinkDatabaseError = 3005,
    /// 短码为空。
    LinkEmptyCode = 3006,
    /// 短码格式非法或超出长度限制。
    LinkInvalidCode = 3007,
    /// 短码命中系统保留字列表。
    LinkReservedCode = 3008,

    // 导入导出错误 4000-4099
    /// 批量导入过程中出现失败。
    ImportFailed = 4000,
    /// 导出流程中途失败。
    ExportFailed = 4001,
    /// multipart 上传数据非法或字段缺失。
    InvalidMultipartData = 4002,
    /// 上传文件内容读取失败。
    FileReadError = 4003,
    /// 上传或指定的 CSV 文件缺失。
    CsvFileMissing = 4004,
    /// CSV 内容无法按预期格式解析。
    CsvParseError = 4005,
    /// 导出 CSV 内容生成失败。
    CsvGenerationError = 4006,

    // 配置错误 5000-5099
    /// 配置 key 未在注册表中声明。
    ConfigNotFound = 5000,
    /// 配置写入、校验或持久化失败。
    ConfigUpdateFailed = 5001,
    /// 配置热重载执行失败。
    ConfigReloadFailed = 5002,

    // Analytics 错误 6000-6099
    /// 分析数据查询执行失败。
    AnalyticsQueryFailed = 6000,
    /// 统计查询的目标短链不存在。
    AnalyticsLinkNotFound = 6001,
    /// 统计查询的起止日期非法或区间无效。
    AnalyticsInvalidDateRange = 6002,
}

#[cfg(all(debug_assertions, feature = "openapi"))]
impl utoipa::PartialSchema for ErrorCode {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        use strum::IntoEnumIterator;
        use utoipa::openapi::extensions::Extensions;
        use utoipa::openapi::schema::{KnownFormat, SchemaFormat, Type};

        let variants = Self::iter().collect::<Vec<_>>();
        let values = variants.iter().map(|variant| *variant as i32);
        let names = variants
            .iter()
            .map(|variant| format!("{variant:?}"))
            .collect::<Vec<_>>();

        utoipa::openapi::ObjectBuilder::new()
            .schema_type(Type::Integer)
            .format(Some(SchemaFormat::KnownFormat(KnownFormat::Int32)))
            .description(Some("Shortlinker API numeric error code"))
            .enum_values(Some(values))
            .extensions(Some(Extensions::from_iter([(
                "x-enum-varnames",
                serde_json::json!(names),
            )])))
            .into()
    }
}

#[cfg(all(debug_assertions, feature = "openapi"))]
impl utoipa::ToSchema for ErrorCode {}
