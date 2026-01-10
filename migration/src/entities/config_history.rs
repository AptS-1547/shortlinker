use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "config_history")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub config_key: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub old_value: Option<String>,
    #[sea_orm(column_type = "Text")]
    pub new_value: String,
    pub changed_at: DateTimeUtc,
    pub changed_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
