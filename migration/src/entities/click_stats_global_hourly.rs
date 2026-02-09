//! 全局小时级点击统计汇总实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "click_stats_global_hourly")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub hour_bucket: DateTimeUtc,
    pub total_clicks: i64,
    pub unique_links: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub top_referrers: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub top_countries: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
