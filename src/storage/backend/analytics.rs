//! Analytics 相关的数据库查询
//!
//! 提供点击日志的统计查询方法，供 AnalyticsService 调用。

use chrono::{DateTime, Utc};
use sea_orm::{
    ColumnTrait, EntityTrait, FromQueryResult, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, sea_query::Expr,
};

use migration::entities::click_log;

// ============ 查询结果类型 ============

/// 趋势查询结果行
#[derive(Debug, FromQueryResult)]
pub struct TrendRow {
    pub label: String,
    pub count: i64,
}

/// 来源查询结果行
#[derive(Debug, FromQueryResult)]
pub struct ReferrerRow {
    pub referrer: Option<String>,
    pub count: i64,
}

/// 地理位置查询结果行
#[derive(Debug, FromQueryResult)]
pub struct GeoRow {
    pub country: Option<String>,
    pub city: Option<String>,
    pub count: i64,
}

/// 热门链接查询结果行
#[derive(Debug, FromQueryResult)]
pub struct TopLinkRow {
    pub short_code: String,
    pub count: i64,
}

// ============ SeaOrmStorage Analytics 方法 ============

impl super::SeaOrmStorage {
    /// 统计指定链接的点击数
    pub async fn count_link_clicks(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<u64> {
        click_log::Entity::find()
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .count(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取链接点击趋势
    pub async fn get_link_trend(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        date_expr: Expr,
    ) -> anyhow::Result<Vec<TrendRow>> {
        click_log::Entity::find()
            .select_only()
            .column_as(date_expr.clone(), "label")
            .column_as(click_log::Column::Id.count(), "count")
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(date_expr)
            .order_by_asc(Expr::cust("label"))
            .into_model::<TrendRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取链接来源统计
    pub async fn get_link_referrers(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<ReferrerRow>> {
        click_log::Entity::find()
            .select_only()
            .column(click_log::Column::Referrer)
            .column_as(click_log::Column::Id.count(), "count")
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(click_log::Column::Referrer)
            .order_by_desc(Expr::cust("count"))
            .limit(limit)
            .into_model::<ReferrerRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取链接地理分布
    pub async fn get_link_geo(
        &self,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<GeoRow>> {
        click_log::Entity::find()
            .select_only()
            .column(click_log::Column::Country)
            .column(click_log::Column::City)
            .column_as(click_log::Column::Id.count(), "count")
            .filter(click_log::Column::ShortCode.eq(code))
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .filter(click_log::Column::Country.is_not_null())
            .group_by(click_log::Column::Country)
            .group_by(click_log::Column::City)
            .order_by_desc(Expr::cust("count"))
            .limit(limit)
            .into_model::<GeoRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    // ============ 全局统计查询（不限定 code） ============

    /// 获取全局点击趋势
    pub async fn get_global_trend(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        date_expr: Expr,
    ) -> anyhow::Result<Vec<TrendRow>> {
        click_log::Entity::find()
            .select_only()
            .column_as(date_expr.clone(), "label")
            .column_as(click_log::Column::Id.count(), "count")
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(date_expr)
            .order_by_asc(Expr::cust("label"))
            .into_model::<TrendRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取热门链接
    pub async fn get_top_links(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<TopLinkRow>> {
        click_log::Entity::find()
            .select_only()
            .column(click_log::Column::ShortCode)
            .column_as(click_log::Column::Id.count(), "count")
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(click_log::Column::ShortCode)
            .order_by_desc(Expr::cust("count"))
            .limit(limit)
            .into_model::<TopLinkRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取全局来源统计
    pub async fn get_global_referrers(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<ReferrerRow>> {
        click_log::Entity::find()
            .select_only()
            .column(click_log::Column::Referrer)
            .column_as(click_log::Column::Id.count(), "count")
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .group_by(click_log::Column::Referrer)
            .order_by_desc(Expr::cust("count"))
            .limit(limit)
            .into_model::<ReferrerRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 获取全局地理分布
    pub async fn get_global_geo(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<GeoRow>> {
        click_log::Entity::find()
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
            .limit(limit)
            .into_model::<GeoRow>()
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    /// 导出点击日志
    pub async fn export_click_logs(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: u64,
    ) -> anyhow::Result<Vec<click_log::Model>> {
        click_log::Entity::find()
            .filter(click_log::Column::ClickedAt.gte(start))
            .filter(click_log::Column::ClickedAt.lte(end))
            .order_by_desc(click_log::Column::ClickedAt)
            .limit(limit)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }
}
