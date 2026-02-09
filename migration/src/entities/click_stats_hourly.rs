//! 小时级点击统计汇总实体

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "click_stats_hourly")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub short_code: String,
    pub hour_bucket: DateTimeUtc,
    pub click_count: i64,
    #[sea_orm(column_type = "Text", nullable)]
    pub referrer_counts: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub country_counts: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub source_counts: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
