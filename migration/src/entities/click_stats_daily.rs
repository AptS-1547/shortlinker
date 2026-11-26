//! 天级点击统计汇总实体

use sea_orm::entity::prelude::*;

/// 单条天级汇总记录；`(short_code, day_bucket)` 唯一。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "click_stats_daily")]
pub struct Model {
    /// 自增主键。
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 该天桶对应的短链接代码。
    pub short_code: String,
    /// 天桶日期（UTC，不含时间部分）。
    pub day_bucket: Date,
    /// 当天该链接的点击总数，默认 0。
    pub click_count: i64,
    /// 当天去重后的来源页面数量（去重 key 计数）；NULL 表示未统计。
    pub unique_referrers: Option<i32>,
    /// 当天去重后的国家数量（去重 key 计数）；NULL 表示未统计。
    pub unique_countries: Option<i32>,
    /// 当天来源页面排行，JSON 数组形式的（名称, 次数）对，取前 10 并按次数降序；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub top_referrers: Option<String>,
    /// 当天国家排行，JSON 数组形式的（国家, 次数）对，取前 10 并按次数降序；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub top_countries: Option<String>,
    /// 当天去重后的流量来源数量（去重 key 计数）；NULL 表示未统计。
    pub unique_sources: Option<i32>,
    /// 当天流量来源排行，JSON 数组形式的（来源, 次数）对，取前 10 并按次数降序；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub top_sources: Option<String>,
}

/// 独立汇总表，不与 `click_logs` 建立声明式关联。
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
