//! UserAgent lookup table entity

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user_agents")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub hash: String, // CHAR(16) xxHash64 hex
    #[sea_orm(column_type = "Text")]
    pub user_agent_string: String,
    pub first_seen: DateTimeUtc,
    pub last_seen: DateTimeUtc,
    // 解析后的字段
    pub browser_name: Option<String>,
    pub browser_version: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub device_category: Option<String>,
    pub device_vendor: Option<String>,
    #[sea_orm(default_value = false)]
    pub is_bot: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
