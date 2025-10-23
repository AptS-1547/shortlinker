use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "short_links")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub short_code: String,
    #[sea_orm(column_type = "Text")]
    pub target_url: String,
    pub created_at: DateTimeUtc,
    pub expires_at: Option<DateTimeUtc>,
    pub password: Option<String>,
    pub click_count: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
