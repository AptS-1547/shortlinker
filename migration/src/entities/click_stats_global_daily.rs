//! 全局天级点击统计汇总实体

use sea_orm::entity::prelude::*;

/// 单条全局天级汇总记录；`day_bucket` 唯一，每天仅一行。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "click_stats_global_daily")]
pub struct Model {
    /// 自增主键。
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 天桶日期（UTC，不含时间部分）。
    pub day_bucket: Date,
    /// 当天全站点击总数，默认 0。
    pub total_clicks: i64,
    /// 当天有点击记录的去重短链接数量；NULL 表示未统计。
    pub unique_links: Option<i32>,
    /// 当天全站来源页面排行，JSON 数组形式的（名称, 次数）对，取前 10 并按次数降序；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub top_referrers: Option<String>,
    /// 当天全站国家排行，JSON 数组形式的（国家, 次数）对，取前 10 并按次数降序；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub top_countries: Option<String>,
    /// 当天全站流量来源排行，JSON 数组形式的（来源, 次数）对，取前 10 并按次数降序；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub top_sources: Option<String>,
}

/// 独立汇总表，不与 `click_logs` 建立声明式关联。
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
