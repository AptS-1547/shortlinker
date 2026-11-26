//! Admin API 类型定义

use serde::{Deserialize, Serialize};

use crate::storage::ShortLink;

// Re-export ValueType from config module
pub use crate::config::ValueType;

/// 管理后台登录凭据
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LoginCredentials {
    /// 管理后台密码明文，只出现在登录请求体中，服务端校验后不会回传
    pub password: String,
}

/// 管理 API 统一响应包装
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ApiResponse<T> {
    /// 业务状态码：0（`ErrorCode::Success`）表示成功，非 0 取值见 `ErrorCode`
    pub code: i32,
    /// 人类可读的状态描述
    pub message: String,
    /// 业务数据载荷；失败或无数据时整个字段省略
    pub data: Option<T>,
}

/// 创建短链接请求体（单条创建与批量创建共用）
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PostNewLink {
    /// 自定义短码；省略或为空时由服务端按短码策略自动生成
    pub code: Option<String>,
    /// 跳转目标 URL，创建和更新时都会经过格式校验
    pub target: String,
    /// 过期时间，接受 `TimeParser` 的灵活格式（RFC3339 或 `1d`/`2w`/`1d2h` 等相对时间）；省略表示永不过期
    pub expires_at: Option<String>,
    /// 链接访问密码明文，服务端入库前统一做哈希；省略表示不启用密码保护
    pub password: Option<String>,
    /// 短码已存在时是否覆盖，默认 `false`（冲突时直接返回错误）
    pub force: Option<bool>,
}

/// 链接列表查询参数（分页与过滤条件）
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(
    all(debug_assertions, feature = "openapi"),
    derive(utoipa::IntoParams, utoipa::ToSchema)
)]
pub struct GetLinksQuery {
    /// 页码（1-based），默认 1
    pub page: Option<usize>,
    /// 每页条数，默认 20，服务端会截断到 1..=100
    pub page_size: Option<usize>,
    /// 只返回该时间之后创建的链接，接受 `TimeParser` 支持的格式
    pub created_after: Option<String>,
    /// 只返回该时间之前创建的链接，接受 `TimeParser` 支持的格式
    pub created_before: Option<String>,
    /// 为 `true` 时只返回已过期的链接
    pub only_expired: Option<bool>,
    /// 为 `true` 时只返回未过期的链接（含无过期时间的链接）
    pub only_active: Option<bool>,
    /// 搜索关键字，对短码和目标 URL 做模糊匹配
    pub search: Option<String>,
}

/// 分页列表的统一响应包装
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PaginatedResponse<T> {
    /// 业务状态码，语义同 `ApiResponse::code`
    pub code: i32,
    /// 人类可读的状态描述
    pub message: String,
    /// 当前页数据
    pub data: Option<T>,
    /// 分页元信息
    pub pagination: PaginationInfo,
}

/// 分页元信息
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct PaginationInfo {
    /// 当前页码（1-based）
    pub page: usize,
    /// 本页实际使用的每页条数
    pub page_size: usize,
    /// 满足过滤条件的记录总数
    pub total: usize,
    /// 按 `total` 与 `page_size` 向上取整得到的总页数
    pub total_pages: usize,
}

// Batch operation request/response types
/// 批量创建短链接请求
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchCreateRequest {
    /// 要创建的链接列表，每项语义同 `PostNewLink`
    pub links: Vec<PostNewLink>,
}

/// 批量更新短链接请求
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchUpdateRequest {
    /// 要更新的链接列表，按短码逐条定位
    pub updates: Vec<BatchUpdateItem>,
}

/// 批量更新中的单条更新项
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchUpdateItem {
    /// 要更新的短码
    pub code: String,
    /// 更新内容，字段语义同创建请求
    pub payload: PostNewLink,
}

/// 批量删除短链接请求
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchDeleteRequest {
    /// 要删除的短码列表，短码不存在时计入失败项
    pub codes: Vec<String>,
}

/// 批量操作结果（部分成功语义：成功与失败条目并列返回）
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchResponse {
    /// 处理成功的短码列表
    pub success: Vec<String>,
    /// 处理失败的条目列表
    pub failed: Vec<BatchFailedItem>,
}

/// 批量操作中的单条失败详情
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct BatchFailedItem {
    /// 操作失败的短码
    pub code: String,
    /// 失败原因描述
    pub error: String,
    /// 预留的业务错误码（`ErrorCode` 数值），当前批量操作总是省略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i32>,
}

/// 短链接详情响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LinkResponse {
    /// 短码
    pub code: String,
    /// 跳转目标 URL
    pub target: String,
    /// 创建时间，RFC 3339 格式
    pub created_at: String,
    /// 过期时间，RFC 3339 格式；`null` 表示永不过期
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub expires_at: Option<String>,
    /// 链接密码的哈希值（不回传明文）；`null` 表示未启用密码保护
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub password: Option<String>,
    /// 该链接累计点击次数
    pub click_count: usize,
}

impl From<ShortLink> for LinkResponse {
    fn from(link: ShortLink) -> Self {
        Self {
            code: link.code,
            target: link.target,
            created_at: link.created_at.to_rfc3339(),
            expires_at: link.expires_at.map(|dt| dt.to_rfc3339()),
            password: link.password,
            click_count: link.click,
        }
    }
}

/// 统计信息响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct StatsResponse {
    /// 短链接总数
    pub total_links: usize,
    /// 所有链接的累计点击次数之和
    pub total_clicks: usize,
    /// 当前未过期的链接数量
    pub active_links: usize,
}

/// 简单消息响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct MessageResponse {
    /// 操作结果描述文本
    pub message: String,
}

/// 认证成功响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct AuthSuccessResponse {
    /// 登录成功的提示文本
    pub message: String,
    /// 会话（JWT）有效期，单位秒
    pub expires_in: u64,
}

/// Reload 成功响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ReloadResponse {
    /// 重载完成的提示文本
    pub message: String,
    /// 重载耗时，单位毫秒
    pub duration_ms: u64,
}

// ============ 健康检查相关类型 ============

/// 存储后端信息
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthStorageBackend {
    /// 存储后端类型标识（如 `sqlite`、`mysql`、`postgres`）
    pub storage_type: String,
    /// 该后端是否支持点击计数写入
    pub support_click: bool,
}

/// 存储健康检查状态
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthStorageCheck {
    /// 存储健康状态，取值 `healthy` 或 `unhealthy`
    pub status: String,
    /// 当前存储的短链接数量；后端无法统计时为 `null`
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub links_count: Option<usize>,
    /// 存储后端描述信息
    pub backend: HealthStorageBackend,
    /// 检查失败时的错误描述，健康时省略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 健康检查项容器
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthChecks {
    /// 存储层检查结果，始终存在
    pub storage: HealthStorageCheck,
    /// 缓存层检查结果；未启用缓存时省略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<HealthCacheCheck>,
}

/// 缓存健康检查状态
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthCacheCheck {
    /// 缓存健康状态，取值 `healthy` 或 `unhealthy`
    pub status: String,
    /// 缓存类型标识（如 `memory`、`redis`、`composite`）
    pub cache_type: String,
    /// 是否启用了 Bloom 存在性过滤器
    pub bloom_filter_enabled: bool,
    /// 是否启用了负缓存（不存在的短码缓存）
    pub negative_cache_enabled: bool,
    /// 检查失败时的错误描述，健康时省略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// 健康检查响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct HealthResponse {
    /// 整体健康状态，取值 `healthy` 或 `unhealthy`
    pub status: String,
    /// 检查完成时刻，RFC 3339 格式
    pub timestamp: String,
    /// 服务进程运行时长，单位秒
    pub uptime: u32,
    /// 各组件检查结果
    pub checks: HealthChecks,
    /// 本次健康检查耗时，单位毫秒
    pub response_time_ms: u32,
}

// ============ 导出导入相关类型 ============

/// 导出查询参数
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[cfg_attr(
    all(debug_assertions, feature = "openapi"),
    derive(utoipa::IntoParams, utoipa::ToSchema)
)]
pub struct ExportQuery {
    /// 只导出短码或目标 URL 模糊匹配该关键字的链接
    pub search: Option<String>,
    /// 只导出该时间之后创建的链接，接受 `TimeParser` 支持的格式
    pub created_after: Option<String>,
    /// 只导出该时间之前创建的链接，接受 `TimeParser` 支持的格式
    pub created_before: Option<String>,
    /// 为 `true` 时只导出已过期的链接
    pub only_expired: Option<bool>,
    /// 为 `true` 时只导出未过期的链接（含无过期时间的链接）
    pub only_active: Option<bool>,
}

/// 导入模式 - 从 service 层 re-export
pub use crate::services::ImportMode;

/// 导入失败项
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ImportFailedItem {
    /// CSV 行号（1-based），None 表示行号未知（如 service 层返回的冲突项无法反查行号）
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub row: Option<usize>,
    /// 导入失败的短码（来自 CSV 的 code 列）
    pub code: String,
    /// 失败原因描述
    pub error: String,
    /// 预留的业务错误码（`ErrorCode` 数值），当前导入流程总是省略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<i32>,
}

/// 导入响应
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ImportResponse {
    /// CSV 中的数据行总数（不含表头）
    pub total_rows: usize,
    /// 成功写入的行数
    pub success_count: usize,
    /// 按导入模式跳过的行数（如 skip 模式下短码已存在）
    pub skipped_count: usize,
    /// 导入失败的行数
    pub failed_count: usize,
    /// 失败行的明细列表
    pub failed_items: Vec<ImportFailedItem>,
}

// Re-export CSV row types from shared csv_handler module
pub use crate::utils::csv_handler::{ClickLogCsvRow, CsvLinkRow};
