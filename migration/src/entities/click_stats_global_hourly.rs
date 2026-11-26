//! 全局小时级点击统计汇总实体

use sea_orm::entity::prelude::*;

/// 单条全局小时级汇总记录；`hour_bucket` 唯一，每小时仅一行。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "click_stats_global_hourly")]
pub struct Model {
    /// 自增主键。
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 小时桶起始时间（UTC，分钟和秒截断为 00）。
    pub hour_bucket: DateTimeUtc,
    /// 该小时内全站点击总数，默认 0。
    pub total_clicks: i64,
    /// 该小时内有点击记录的去重短链接数量；NULL 表示未统计。
    pub unique_links: Option<i32>,
    /// 该小时内全站来源页面排行，JSON 数组形式的（名称, 次数）对，取前 10 并按次数降序；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub top_referrers: Option<String>,
    /// 该小时内全站国家排行，JSON 数组形式的（国家, 次数）对，取前 10 并按次数降序；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub top_countries: Option<String>,
}

/// 独立汇总表，不与 `click_logs` 建立声明式关联。
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
