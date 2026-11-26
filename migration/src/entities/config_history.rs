//! 配置变更历史实体
//!
//! 对应 `config_history` 表，记录每次配置值修改的前后值。

use sea_orm::entity::prelude::*;

/// 一条配置变更历史记录，每次修改追加一行。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "config_history")]
pub struct Model {
    /// 自增主键。
    #[sea_orm(primary_key)]
    pub id: i32,
    /// 发生变更的配置 key。
    pub config_key: String,
    /// 变更前的值（字符串形式）；NULL 表示首次创建该配置项。
    #[sea_orm(column_type = "Text", nullable)]
    pub old_value: Option<String>,
    /// 变更后的值（字符串形式）。
    #[sea_orm(column_type = "Text")]
    pub new_value: String,
    /// 变更时间（UTC）。
    pub changed_at: DateTimeUtc,
    /// 操作者标识；NULL 表示未记录（如系统初始化写入）。
    pub changed_by: Option<String>,
}

/// 独立实体，未声明与其他表的关联。
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
