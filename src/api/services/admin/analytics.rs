//! Analytics API 端点
//!
//! 提供详细的点击统计分析功能：
//! - 点击趋势（按日期/小时分组）
//! - 热门链接排行
//! - 来源统计
//! - 地理位置分布
//! - 单链接详细统计
//! - 导出报告

use actix_web::{HttpRequest, HttpResponse, web};
use chrono::{DateTime, Duration, Utc};
use sea_orm::{
    ColumnTrait, DbBackend, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, sea_query::Expr,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;
use ts_rs::TS;

use crate::api::constants;
use crate::api::jwt::JwtService;
use crate::api::services::admin::{ErrorCode, error_response, success_response};
use crate::storage::SeaOrmStorage;
use actix_web::http::StatusCode;
use migration::entities::click_log;

/// 输出目录常量
const TS_EXPORT_PATH: &str = "../admin-panel/src/services/types.generated.ts";

// ============ 请求参数 ============

/// Analytics 查询参数
#[derive(Debug, Clone, Deserialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize, TS, Default)]
#[ts(export, export_to = TS_EXPORT_PATH)]
#[serde(rename_all = "lowercase")]
pub enum GroupBy {
    Hour,
    #[default]
    Day,
    Week,
    Month,
}

// ============ 响应结构 ============

/// 点击趋势数据
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct TrendData {
    /// 时间标签
    pub labels: Vec<String>,
    /// 点击数
    pub values: Vec<u64>,
}

/// 热门链接
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct TopLink {
    pub code: String,
    pub clicks: u64,
}

/// 来源统计
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct ReferrerStats {
    pub referrer: String,
    pub count: u64,
    pub percentage: f64,
}

/// 地理位置统计
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct GeoStats {
    pub country: String,
    pub city: Option<String>,
    pub count: u64,
}

/// 单链接分析数据
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = TS_EXPORT_PATH)]
pub struct LinkAnalytics {
    pub code: String,
    pub total_clicks: u64,
    pub trend: TrendData,
    pub top_referrers: Vec<ReferrerStats>,
    pub geo_distribution: Vec<GeoStats>,
}

// ============ 辅助函数 ============

fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|dt| dt.and_utc())
        })
}

fn default_date_range() -> (DateTime<Utc>, DateTime<Utc>) {
    let end = Utc::now();
    let start = end - Duration::days(30);
    (start, end)
}

/// 验证 JWT（从 Cookie 或 Bearer Token）
fn verify_jwt(req: &HttpRequest) -> bool {
    // 1. 尝试 Bearer Token
    if let Some(auth_header) = req.headers().get("Authorization")
        && let Ok(auth_str) = auth_header.to_str()
        && let Some(token) = auth_str.strip_prefix("Bearer ")
    {
        let jwt_service = JwtService::from_config();
        if jwt_service.validate_access_token(token).is_ok() {
            return true;
        }
    }

    // 2. 尝试 Cookie
    if let Some(cookie) = req.cookie(constants::ACCESS_COOKIE_NAME) {
        let jwt_service = JwtService::from_config();
        if jwt_service.validate_access_token(cookie.value()).is_ok() {
            return true;
        }
    }

    false
}

// ============ SeaORM DSL 辅助结构 ============

/// 趋势查询结果
#[derive(Debug, FromQueryResult)]
struct TrendResult {
    label: String,
    count: i64,
}

/// 热门链接查询结果
#[derive(Debug, FromQueryResult)]
struct TopLinkResult {
    short_code: String,
    count: i64,
}

/// 来源查询结果
#[derive(Debug, FromQueryResult)]
struct ReferrerResult {
    referrer: Option<String>,
    count: i64,
}

/// 地理位置查询结果
#[derive(Debug, FromQueryResult)]
struct GeoResult {
    country: Option<String>,
    city: Option<String>,
    count: i64,
}

/// 获取数据库后端类型
fn get_db_backend(storage: &SeaOrmStorage) -> DbBackend {
    match storage.get_backend_name() {
        "sqlite" => DbBackend::Sqlite,
        "mysql" => DbBackend::MySql,
        _ => DbBackend::Postgres,
    }
}

/// 根据数据库类型和分组方式生成日期格式化表达式
fn date_format_expr(backend: DbBackend, group_by: GroupBy) -> Expr {
    let (sqlite_fmt, mysql_fmt, pg_fmt) = match group_by {
        GroupBy::Hour => ("%Y-%m-%d %H:00", "%Y-%m-%d %H:00", "YYYY-MM-DD HH24:00"),
        GroupBy::Day => ("%Y-%m-%d", "%Y-%m-%d", "YYYY-MM-DD"),
        GroupBy::Week => ("%Y-W%W", "%Y-W%U", "IYYY-\"W\"IW"),
        GroupBy::Month => ("%Y-%m", "%Y-%m", "YYYY-MM"),
    };

    match backend {
        DbBackend::Sqlite => Expr::cust(format!("strftime('{}', clicked_at)", sqlite_fmt)),
        DbBackend::MySql => Expr::cust(format!("DATE_FORMAT(clicked_at, '{}')", mysql_fmt)),
        DbBackend::Postgres | _ => Expr::cust(format!("TO_CHAR(clicked_at, '{}')", pg_fmt)),
    }
}

// ============ API 端点 ============

/// GET /admin/v1/analytics/trends - 获取点击趋势
pub async fn get_trends(
    req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> actix_web::Result<HttpResponse> {
    if !verify_jwt(&req) {
        return Ok(error_response(
            StatusCode::UNAUTHORIZED,
            ErrorCode::Unauthorized,
            "Unauthorized",
        ));
    }

    let (start, end) = match (&query.start_date, &query.end_date) {
        (Some(s), Some(e)) => {
            let start = parse_date(s).unwrap_or_else(|| default_date_range().0);
            let end = parse_date(e).unwrap_or_else(|| default_date_range().1);
            (start, end)
        }
        _ => default_date_range(),
    };

    let group_by = query.group_by.unwrap_or_default();
    let db = storage.get_db();
    let backend = get_db_backend(&storage);
    let date_expr = date_format_expr(backend, group_by);

    let results = click_log::Entity::find()
        .select_only()
        .column_as(date_expr.clone(), "label")
        .column_as(click_log::Column::Id.count(), "count")
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .group_by(date_expr)
        .order_by_asc(Expr::cust("label"))
        .into_model::<TrendResult>()
        .all(db)
        .await;

    let results = match results {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to query trends: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
                "Database error",
            ));
        }
    };

    let mut labels = Vec::new();
    let mut values = Vec::new();
    for row in results {
        labels.push(row.label);
        values.push(row.count as u64);
    }

    let trend = TrendData { labels, values };
    Ok(success_response(trend))
}

/// GET /admin/v1/analytics/top - 获取热门链接
pub async fn get_top_links(
    req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> actix_web::Result<HttpResponse> {
    if !verify_jwt(&req) {
        return Ok(error_response(
            StatusCode::UNAUTHORIZED,
            ErrorCode::Unauthorized,
            "Unauthorized",
        ));
    }

    let limit = query.limit.unwrap_or(10).min(100);
    let (start, end) = match (&query.start_date, &query.end_date) {
        (Some(s), Some(e)) => {
            let start = parse_date(s).unwrap_or_else(|| default_date_range().0);
            let end = parse_date(e).unwrap_or_else(|| default_date_range().1);
            (start, end)
        }
        _ => default_date_range(),
    };

    let db = storage.get_db();

    let results = click_log::Entity::find()
        .select_only()
        .column(click_log::Column::ShortCode)
        .column_as(click_log::Column::Id.count(), "count")
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .group_by(click_log::Column::ShortCode)
        .order_by_desc(Expr::cust("count"))
        .limit(limit as u64)
        .into_model::<TopLinkResult>()
        .all(db)
        .await;

    let results = match results {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to query top links: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
                "Database error",
            ));
        }
    };

    let top_links: Vec<TopLink> = results
        .into_iter()
        .map(|row| TopLink {
            code: row.short_code,
            clicks: row.count as u64,
        })
        .collect();

    Ok(success_response(top_links))
}

/// GET /admin/v1/analytics/referrers - 获取来源统计
pub async fn get_referrers(
    req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> actix_web::Result<HttpResponse> {
    if !verify_jwt(&req) {
        return Ok(error_response(
            StatusCode::UNAUTHORIZED,
            ErrorCode::Unauthorized,
            "Unauthorized",
        ));
    }

    let limit = query.limit.unwrap_or(10).min(100);
    let (start, end) = match (&query.start_date, &query.end_date) {
        (Some(s), Some(e)) => {
            let start = parse_date(s).unwrap_or_else(|| default_date_range().0);
            let end = parse_date(e).unwrap_or_else(|| default_date_range().1);
            (start, end)
        }
        _ => default_date_range(),
    };

    let db = storage.get_db();

    // 获取总数
    let total: u64 = click_log::Entity::find()
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .count(db)
        .await
        .unwrap_or(0);

    let results = click_log::Entity::find()
        .select_only()
        .column(click_log::Column::Referrer)
        .column_as(click_log::Column::Id.count(), "count")
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .group_by(click_log::Column::Referrer)
        .order_by_desc(Expr::cust("count"))
        .limit(limit as u64)
        .into_model::<ReferrerResult>()
        .all(db)
        .await;

    let results = match results {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to query referrers: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
                "Database error",
            ));
        }
    };

    let referrer_stats: Vec<ReferrerStats> = results
        .into_iter()
        .map(|row| {
            let referrer = row.referrer.unwrap_or_else(|| "(direct)".to_string());
            let count = row.count as u64;
            let percentage = if total > 0 {
                (count as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            ReferrerStats {
                referrer,
                count,
                percentage,
            }
        })
        .collect();

    Ok(success_response(referrer_stats))
}

/// GET /admin/v1/analytics/geo - 获取地理位置分布
pub async fn get_geo_stats(
    req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> actix_web::Result<HttpResponse> {
    if !verify_jwt(&req) {
        return Ok(error_response(
            StatusCode::UNAUTHORIZED,
            ErrorCode::Unauthorized,
            "Unauthorized",
        ));
    }

    let limit = query.limit.unwrap_or(20).min(100);
    let (start, end) = match (&query.start_date, &query.end_date) {
        (Some(s), Some(e)) => {
            let start = parse_date(s).unwrap_or_else(|| default_date_range().0);
            let end = parse_date(e).unwrap_or_else(|| default_date_range().1);
            (start, end)
        }
        _ => default_date_range(),
    };

    let db = storage.get_db();

    let results = click_log::Entity::find()
        .select_only()
        .column(click_log::Column::Country)
        .column(click_log::Column::City)
        .column_as(click_log::Column::Id.count(), "count")
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .filter(click_log::Column::Country.is_not_null())
        .group_by(click_log::Column::Country)
        .group_by(click_log::Column::City)
        .order_by_desc(Expr::cust("count"))
        .limit(limit as u64)
        .into_model::<GeoResult>()
        .all(db)
        .await;

    let results = match results {
        Ok(rows) => rows,
        Err(e) => {
            error!("Failed to query geo stats: {}", e);
            return Ok(error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalServerError,
                "Database error",
            ));
        }
    };

    let geo_stats: Vec<GeoStats> = results
        .into_iter()
        .map(|row| GeoStats {
            country: row.country.unwrap_or_else(|| "Unknown".to_string()),
            city: row.city,
            count: row.count as u64,
        })
        .collect();

    Ok(success_response(geo_stats))
}

/// GET /admin/v1/links/{code}/analytics - 获取单链接详细统计
pub async fn get_link_analytics(
    req: HttpRequest,
    code: web::Path<String>,
    query: web::Query<AnalyticsQuery>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> actix_web::Result<HttpResponse> {
    if !verify_jwt(&req) {
        return Ok(error_response(
            StatusCode::UNAUTHORIZED,
            ErrorCode::Unauthorized,
            "Unauthorized",
        ));
    }

    let code = code.into_inner();
    let (start, end) = match (&query.start_date, &query.end_date) {
        (Some(s), Some(e)) => {
            let start = parse_date(s).unwrap_or_else(|| default_date_range().0);
            let end = parse_date(e).unwrap_or_else(|| default_date_range().1);
            (start, end)
        }
        _ => default_date_range(),
    };

    let db = storage.get_db();
    let backend = get_db_backend(&storage);

    // 总点击数
    let total_clicks: u64 = click_log::Entity::find()
        .filter(click_log::Column::ShortCode.eq(&code))
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .count(db)
        .await
        .unwrap_or(0);

    // 趋势数据 - 使用 Day 分组
    let date_expr = date_format_expr(backend, GroupBy::Day);

    let trend_results = click_log::Entity::find()
        .select_only()
        .column_as(date_expr.clone(), "label")
        .column_as(click_log::Column::Id.count(), "count")
        .filter(click_log::Column::ShortCode.eq(&code))
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .group_by(date_expr)
        .order_by_asc(Expr::cust("label"))
        .into_model::<TrendResult>()
        .all(db)
        .await
        .unwrap_or_default();

    let mut labels = Vec::new();
    let mut values = Vec::new();
    for row in trend_results {
        labels.push(row.label);
        values.push(row.count as u64);
    }
    let trend = TrendData { labels, values };

    // 来源统计
    let referrer_results = click_log::Entity::find()
        .select_only()
        .column(click_log::Column::Referrer)
        .column_as(click_log::Column::Id.count(), "count")
        .filter(click_log::Column::ShortCode.eq(&code))
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .group_by(click_log::Column::Referrer)
        .order_by_desc(Expr::cust("count"))
        .limit(10)
        .into_model::<ReferrerResult>()
        .all(db)
        .await
        .unwrap_or_default();

    let top_referrers: Vec<ReferrerStats> = referrer_results
        .into_iter()
        .map(|row| {
            let referrer = row.referrer.unwrap_or_else(|| "(direct)".to_string());
            let count = row.count as u64;
            let percentage = if total_clicks > 0 {
                (count as f64 / total_clicks as f64) * 100.0
            } else {
                0.0
            };
            ReferrerStats {
                referrer,
                count,
                percentage,
            }
        })
        .collect();

    // 地理分布
    let geo_results = click_log::Entity::find()
        .select_only()
        .column(click_log::Column::Country)
        .column(click_log::Column::City)
        .column_as(click_log::Column::Id.count(), "count")
        .filter(click_log::Column::ShortCode.eq(&code))
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .filter(click_log::Column::Country.is_not_null())
        .group_by(click_log::Column::Country)
        .group_by(click_log::Column::City)
        .order_by_desc(Expr::cust("count"))
        .limit(10)
        .into_model::<GeoResult>()
        .all(db)
        .await
        .unwrap_or_default();

    let geo_distribution: Vec<GeoStats> = geo_results
        .into_iter()
        .map(|row| GeoStats {
            country: row.country.unwrap_or_else(|| "Unknown".to_string()),
            city: row.city,
            count: row.count as u64,
        })
        .collect();

    let analytics = LinkAnalytics {
        code,
        total_clicks,
        trend,
        top_referrers,
        geo_distribution,
    };

    Ok(success_response(analytics))
}

/// GET /admin/v1/analytics/export - 导出报告
pub async fn export_report(
    req: HttpRequest,
    query: web::Query<AnalyticsQuery>,
    storage: web::Data<Arc<SeaOrmStorage>>,
) -> actix_web::Result<HttpResponse> {
    if !verify_jwt(&req) {
        return Ok(error_response(
            StatusCode::UNAUTHORIZED,
            ErrorCode::Unauthorized,
            "Unauthorized",
        ));
    }

    let (start, end) = match (&query.start_date, &query.end_date) {
        (Some(s), Some(e)) => {
            let start = parse_date(s).unwrap_or_else(|| default_date_range().0);
            let end = parse_date(e).unwrap_or_else(|| default_date_range().1);
            (start, end)
        }
        _ => default_date_range(),
    };

    let db = storage.get_db();

    // 查询所有日志
    let logs = click_log::Entity::find()
        .filter(click_log::Column::ClickedAt.gte(start))
        .filter(click_log::Column::ClickedAt.lte(end))
        .order_by_desc(click_log::Column::ClickedAt)
        .all(db)
        .await
        .unwrap_or_default();

    // 生成 CSV
    let mut csv_content =
        String::from("short_code,clicked_at,referrer,user_agent,ip_address,country,city\n");
    for log in logs {
        csv_content.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            log.short_code,
            log.clicked_at.to_rfc3339(),
            log.referrer.as_deref().unwrap_or(""),
            log.user_agent.as_deref().unwrap_or("").replace(',', ";"),
            log.ip_address.as_deref().unwrap_or(""),
            log.country.as_deref().unwrap_or(""),
            log.city.as_deref().unwrap_or(""),
        ));
    }

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
        .body(csv_content))
}

/// Analytics 路由配置
pub fn analytics_routes() -> actix_web::Scope {
    web::scope("/analytics")
        .route("/trends", web::get().to(get_trends))
        .route("/top", web::get().to(get_top_links))
        .route("/referrers", web::get().to(get_referrers))
        .route("/geo", web::get().to(get_geo_stats))
        .route("/export", web::get().to(export_report))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_typescript_types() {
        AnalyticsQuery::export_all().expect("Failed to export AnalyticsQuery");
        GroupBy::export_all().expect("Failed to export GroupBy");
        TrendData::export_all().expect("Failed to export TrendData");
        TopLink::export_all().expect("Failed to export TopLink");
        ReferrerStats::export_all().expect("Failed to export ReferrerStats");
        GeoStats::export_all().expect("Failed to export GeoStats");
        LinkAnalytics::export_all().expect("Failed to export LinkAnalytics");
        println!("Analytics TypeScript types exported to {}", TS_EXPORT_PATH);
    }
}
