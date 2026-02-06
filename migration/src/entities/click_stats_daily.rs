//! 天级点击统计汇总实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "click_stats_daily")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub short_code: String,
    pub day_bucket: Date,
    pub click_count: i64,
    pub unique_referrers: Option<i32>,
    pub unique_countries: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub top_referrers: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub top_countries: Option<String>,
    pub unique_sources: Option<i32>,
    #[sea_orm(column_type = "Text", nullable)]
    pub top_sources: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
