//! Analytics API 端点
//!
//! 提供详细的点击统计分析功能：
//! - 点击趋势（按日期/小时分组）
//! - 热门链接排行
//! - 来源统计
//! - 地理位置分布
//! - 单链接详细统计
//! - 导出报告

use actix_web::{HttpRequest, HttpResponse, Responder, Result as ActixResult, web};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::services::{
    AnalyticsService, CategoryStats as ServiceCategoryStats,
    DeviceAnalytics as ServiceDeviceAnalytics, GeoStats as ServiceGeoStats,
    GroupBy as ServiceGroupBy, LinkAnalytics as ServiceLinkAnalytics,
    ReferrerStats as ServiceReferrerStats, TopLink as ServiceTopLink,
    TrendData as ServiceTrendData,
};

use super::export_import::create_csv_stream;
use super::helpers::{error_from_shortlinker, success_response};
use super::types::ClickLogCsvRow;

// ============ 请求参数 ============

/// Analytics 查询参数
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(
    all(debug_assertions, feature = "openapi"),
    derive(utoipa::IntoParams, utoipa::ToSchema)
)]
pub struct AnalyticsQuery {
    /// 开始日期 (ISO 8601)
    pub start_date: Option<String>,
    /// 结束日期 (ISO 8601)
    pub end_date: Option<String>,
    /// 分组方式
    pub group_by: Option<GroupBy>,
    /// 返回数量限制
    pub limit: Option<u32>,
}

/// 分组方式
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
#[serde(rename_all = "lowercase")]
pub enum GroupBy {
    /// 按小时分组，标签形如 `2026-01-01 08:00`
    Hour,
    /// 按天分组（默认值），标签形如 `2026-01-01`
    #[default]
    Day,
    /// 按周分组，标签形如 `2026-W02`
    Week,
    /// 按月分组，标签形如 `2026-01`
    Month,
}

impl From<GroupBy> for ServiceGroupBy {
    fn from(g: GroupBy) -> Self {
        match g {
            GroupBy::Hour => ServiceGroupBy::Hour,
            GroupBy::Day => ServiceGroupBy::Day,
            GroupBy::Week => ServiceGroupBy::Week,
            GroupBy::Month => ServiceGroupBy::Month,
        }
    }
}

// ============ 响应结构 ============

/// 点击趋势数据
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct TrendData {
    /// 时间标签
    pub labels: Vec<String>,
    /// 点击数
    pub values: Vec<u64>,
}

impl From<ServiceTrendData> for TrendData {
    fn from(t: ServiceTrendData) -> Self {
        TrendData {
            labels: t.labels,
            values: t.values,
        }
    }
}

/// 热门链接
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct TopLink {
    /// 短码
    pub code: String,
    /// 统计区间内的点击次数
    pub clicks: u64,
}

impl From<ServiceTopLink> for TopLink {
    fn from(t: ServiceTopLink) -> Self {
        TopLink {
            code: t.code,
            clicks: t.clicks,
        }
    }
}

/// 来源统计
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct ReferrerStats {
    /// 来源页 URL 或主机；直接访问（无 Referer）时为 `(direct)`
    pub referrer: String,
    /// 该来源的点击次数
    pub count: u64,
    /// 占本次返回的各来源点击数之和的百分比（0-100）
    pub percentage: f64,
}

impl From<ServiceReferrerStats> for ReferrerStats {
    fn from(r: ServiceReferrerStats) -> Self {
        ReferrerStats {
            referrer: r.referrer,
            count: r.count,
            percentage: r.percentage,
        }
    }
}

/// 地理位置统计
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct GeoStats {
    /// 国家/地区名称，来自 GeoIP 解析；无法解析时为 `Unknown`
    pub country: String,
    /// 城市名称；GeoIP 未命中城市信息时为 `null`
    #[cfg_attr(all(debug_assertions, feature = "openapi"), schema(required))]
    pub city: Option<String>,
    /// 该地区的点击次数
    pub count: u64,
}

impl From<ServiceGeoStats> for GeoStats {
    fn from(g: ServiceGeoStats) -> Self {
        GeoStats {
            country: g.country,
            city: g.city,
            count: g.count,
        }
    }
}

/// 单链接分析数据
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct LinkAnalytics {
    /// 短码
    pub code: String,
    /// 统计区间内的点击总数
    pub total_clicks: u64,
    /// 区间内按天分组的点击趋势
    pub trend: TrendData,
    /// 点击次数最高的前 10 个来源
    pub top_referrers: Vec<ReferrerStats>,
    /// 点击次数最高的前 10 个地区分布
    pub geo_distribution: Vec<GeoStats>,
}

impl From<ServiceLinkAnalytics> for LinkAnalytics {
    fn from(l: ServiceLinkAnalytics) -> Self {
        LinkAnalytics {
            code: l.code,
            total_clicks: l.total_clicks,
            trend: l.trend.into(),
            top_referrers: l.top_referrers.into_iter().map(Into::into).collect(),
            geo_distribution: l.geo_distribution.into_iter().map(Into::into).collect(),
        }
    }
}

/// 设备分析响应
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct DeviceAnalyticsResponse {
    /// 浏览器维度统计，按点击次数降序
    pub browsers: Vec<CategoryStatsResponse>,
    /// 操作系统维度统计，按点击次数降序
    pub operating_systems: Vec<CategoryStatsResponse>,
    /// 设备类型维度统计，按点击次数降序
    pub devices: Vec<CategoryStatsResponse>,
    /// 点击中 Bot（爬虫/自动化 UA）占比（0-100）
    pub bot_percentage: f64,
    /// 统计区间内带 User-Agent 信息的点击总数，是各分类百分比的分母
    pub total_with_ua: u64,
}

/// 分类统计响应
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(all(debug_assertions, feature = "openapi"), derive(utoipa::ToSchema))]
pub struct CategoryStatsResponse {
    /// 分类名称（如浏览器名 `Chrome`、系统名 `Windows`、设备类型 `mobile`）；UA 缺失时为 `Unknown`
    pub name: String,
    /// 该分类的点击次数
    pub count: u64,
    /// 占同维度各分类点击数之和的百分比（0-100）
    pub percentage: f64,
}

impl From<ServiceDeviceAnalytics> for DeviceAnalyticsResponse {
    fn from(d: ServiceDeviceAnalytics) -> Self {
        DeviceAnalyticsResponse {
            browsers: d.browsers.into_iter().map(Into::into).collect(),
            operating_systems: d.operating_systems.into_iter().map(Into::into).collect(),
            devices: d.devices.into_iter().map(Into::into).collect(),
            bot_percentage: d.bot_percentage,
            total_with_ua: d.total_with_ua,
        }
    }
}

impl From<ServiceCategoryStats> for CategoryStatsResponse {
    fn from(c: ServiceCategoryStats) -> Self {
        CategoryStatsResponse {
            name: c.name,
            count: c.count,
            percentage: c.percentage,
        }
    }
}

// ============ API 端点 ============

/// GET /admin/v1/analytics/trends - 获取点击趋势
#[aster_forge_api_docs_macros::path(
        get,
        path = "/admin/v1/analytics/trends",
        tag = "analytics",
        operation_id = "get_analytics_trends",
        params(AnalyticsQuery),
        responses((status = 200, description = "Click trend", body = super::types::ApiResponse<TrendData>))
)]
pub async fn get_trends(
    _req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    service: web::Data<Arc<AnalyticsService>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: get_trends with query: {:?}", query);

    let (start, end) =
        AnalyticsService::parse_date_range(query.start_date.as_deref(), query.end_date.as_deref());
    let group_by: ServiceGroupBy = query.group_by.unwrap_or_default().into();

    match service.get_trends(start, end, group_by).await {
        Ok(trend) => Ok(success_response(TrendData::from(trend))),
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// GET /admin/v1/analytics/top - 获取热门链接
#[aster_forge_api_docs_macros::path(
        get,
        path = "/admin/v1/analytics/top",
        tag = "analytics",
        operation_id = "get_top_links",
        params(AnalyticsQuery),
        responses((status = 200, description = "Top links", body = super::types::ApiResponse<Vec<TopLink>>))
)]
pub async fn get_top_links(
    _req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    service: web::Data<Arc<AnalyticsService>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: get_top_links with query: {:?}", query);

    let (start, end) =
        AnalyticsService::parse_date_range(query.start_date.as_deref(), query.end_date.as_deref());
    let limit = query.limit.unwrap_or(10);

    match service.get_top_links(start, end, limit).await {
        Ok(links) => {
            let response: Vec<TopLink> = links.into_iter().map(Into::into).collect();
            Ok(success_response(response))
        }
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// GET /admin/v1/analytics/referrers - 获取来源统计
#[aster_forge_api_docs_macros::path(
        get,
        path = "/admin/v1/analytics/referrers",
        tag = "analytics",
        operation_id = "get_referrer_stats",
        params(AnalyticsQuery),
        responses((status = 200, description = "Referrer statistics", body = super::types::ApiResponse<Vec<ReferrerStats>>))
)]
pub async fn get_referrers(
    _req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    service: web::Data<Arc<AnalyticsService>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: get_referrers with query: {:?}", query);

    let (start, end) =
        AnalyticsService::parse_date_range(query.start_date.as_deref(), query.end_date.as_deref());
    let limit = query.limit.unwrap_or(10);

    match service.get_referrers(start, end, limit).await {
        Ok(referrers) => {
            let response: Vec<ReferrerStats> = referrers.into_iter().map(Into::into).collect();
            Ok(success_response(response))
        }
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// GET /admin/v1/analytics/geo - 获取地理位置分布
#[aster_forge_api_docs_macros::path(
        get,
        path = "/admin/v1/analytics/geo",
        tag = "analytics",
        operation_id = "get_geo_stats",
        params(AnalyticsQuery),
        responses((status = 200, description = "Geographic statistics", body = super::types::ApiResponse<Vec<GeoStats>>))
)]
pub async fn get_geo_stats(
    _req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    service: web::Data<Arc<AnalyticsService>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: get_geo_stats with query: {:?}", query);

    let (start, end) =
        AnalyticsService::parse_date_range(query.start_date.as_deref(), query.end_date.as_deref());
    let limit = query.limit.unwrap_or(20);

    match service.get_geo_stats(start, end, limit).await {
        Ok(geo) => {
            let response: Vec<GeoStats> = geo.into_iter().map(Into::into).collect();
            Ok(success_response(response))
        }
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// GET /admin/v1/links/{code}/analytics - 获取单链接详细统计
#[aster_forge_api_docs_macros::path(
        get,
        path = "/admin/v1/links/{code}/analytics",
        tag = "analytics",
        operation_id = "get_link_analytics",
        params(
            ("code" = String, Path, description = "Short code"),
            AnalyticsQuery,
        ),
        responses(
            (status = 200, description = "Link analytics", body = super::types::ApiResponse<LinkAnalytics>),
            (status = 404, description = "Short link not found"),
        )
)]
pub async fn get_link_analytics(
    _req: HttpRequest,
    code: web::Path<String>,
    query: web::Query<AnalyticsQuery>,
    service: web::Data<Arc<AnalyticsService>>,
) -> ActixResult<impl Responder> {
    let code = code.into_inner();
    info!(
        "Admin API: get_link_analytics for '{}' with query: {:?}",
        code, query
    );

    let (start, end) =
        AnalyticsService::parse_date_range(query.start_date.as_deref(), query.end_date.as_deref());

    match service.get_link_analytics(&code, start, end).await {
        Ok(analytics) => Ok(success_response(LinkAnalytics::from(analytics))),
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// GET /admin/v1/links/{code}/analytics/devices - 获取单链接设备分析
#[aster_forge_api_docs_macros::path(
        get,
        path = "/admin/v1/links/{code}/analytics/devices",
        tag = "analytics",
        operation_id = "get_link_device_stats",
        params(
            ("code" = String, Path, description = "Short code"),
            AnalyticsQuery,
        ),
        responses(
            (status = 200, description = "Link device statistics", body = super::types::ApiResponse<DeviceAnalyticsResponse>),
            (status = 404, description = "Short link not found"),
        )
)]
pub async fn get_link_device_stats(
    _req: HttpRequest,
    code: web::Path<String>,
    query: web::Query<AnalyticsQuery>,
    service: web::Data<Arc<AnalyticsService>>,
) -> ActixResult<impl Responder> {
    let code = code.into_inner();
    info!(
        "Admin API: get_link_device_stats for '{}' with query: {:?}",
        code, query
    );

    let (start, end) =
        AnalyticsService::parse_date_range(query.start_date.as_deref(), query.end_date.as_deref());
    let limit = query.limit.unwrap_or(10);

    match service
        .get_link_device_analytics(&code, start, end, limit)
        .await
    {
        Ok(analytics) => Ok(success_response(DeviceAnalyticsResponse::from(analytics))),
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// GET /admin/v1/analytics/devices - 获取设备分析
#[aster_forge_api_docs_macros::path(
        get,
        path = "/admin/v1/analytics/devices",
        tag = "analytics",
        operation_id = "get_device_stats",
        params(AnalyticsQuery),
        responses((status = 200, description = "Device statistics", body = super::types::ApiResponse<DeviceAnalyticsResponse>))
)]
pub async fn get_device_stats(
    _req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    service: web::Data<Arc<AnalyticsService>>,
) -> ActixResult<impl Responder> {
    info!("Admin API: get_device_stats with query: {:?}", query);

    let (start, end) =
        AnalyticsService::parse_date_range(query.start_date.as_deref(), query.end_date.as_deref());
    let limit = query.limit.unwrap_or(10);

    match service.get_device_analytics(start, end, limit).await {
        Ok(analytics) => Ok(success_response(DeviceAnalyticsResponse::from(analytics))),
        Err(e) => Ok(error_from_shortlinker(&e)),
    }
}

/// 每批次导出的日志数量
const EXPORT_BATCH_SIZE: u64 = 10000;

/// GET /admin/v1/analytics/export - 导出报告（流式响应）
#[aster_forge_api_docs_macros::path(
        get,
        path = "/admin/v1/analytics/export",
        tag = "analytics",
        operation_id = "export_analytics_report",
        params(AnalyticsQuery),
        responses((status = 200, description = "Analytics CSV report", content_type = "text/csv"))
)]
pub async fn export_report(
    _req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    service: web::Data<Arc<AnalyticsService>>,
) -> ActixResult<impl Responder> {
    info!(
        "Admin API: export_report (streaming) with query: {:?}",
        query
    );

    let (start, end) =
        AnalyticsService::parse_date_range(query.start_date.as_deref(), query.end_date.as_deref());

    // 获取流式数据
    let batch_stream = service.export_click_logs_stream(start, end, EXPORT_BATCH_SIZE);

    // 行映射：click_log::Model → ClickLogCsvRow
    let row_mapper = |log: migration::entities::click_log::Model| ClickLogCsvRow {
        short_code: log.short_code,
        clicked_at: log.clicked_at.to_rfc3339(),
        referrer: log.referrer.unwrap_or_default(),
        source: log.source.unwrap_or_default(),
        ip_address: log.ip_address.unwrap_or_default(),
        country: log.country.unwrap_or_default(),
        city: log.city.unwrap_or_default(),
    };

    // 创建 CSV 流
    let csv_stream = create_csv_stream(batch_stream, row_mapper, "logs");

    // 返回流式响应
    Ok(HttpResponse::Ok()
        .content_type("text/csv; charset=utf-8")
        .insert_header((
            "Content-Disposition",
            format!(
                "attachment; filename=\"analytics_{}_{}.csv\"",
                start.format("%Y%m%d"),
                end.format("%Y%m%d")
            ),
        ))
        .insert_header(("Transfer-Encoding", "chunked"))
        .streaming(csv_stream))
}

/// Analytics 路由配置
pub fn analytics_routes() -> actix_web::Scope {
    web::scope("/analytics")
        .route("/trends", web::get().to(get_trends))
        .route("/trends", web::head().to(get_trends))
        .route("/top", web::get().to(get_top_links))
        .route("/top", web::head().to(get_top_links))
        .route("/referrers", web::get().to(get_referrers))
        .route("/referrers", web::head().to(get_referrers))
        .route("/geo", web::get().to(get_geo_stats))
        .route("/geo", web::head().to(get_geo_stats))
        .route("/devices", web::get().to(get_device_stats))
        .route("/devices", web::head().to(get_device_stats))
        .route("/export", web::get().to(export_report))
        .route("/export", web::head().to(export_report))
}
