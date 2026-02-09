//! Click log entity for detailed click tracking

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "click_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub short_code: String,
    pub clicked_at: DateTimeUtc,
    #[sea_orm(column_type = "Text", nullable)]
    pub referrer: Option<String>,
    pub ip_address: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
    /// Traffic source (utm_source param, ref:{domain}, or direct)
    pub source: Option<String>,
    /// UserAgent hash (references user_agents.hash)
    pub user_agent_hash: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
