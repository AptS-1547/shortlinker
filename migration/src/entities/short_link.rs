//! 短链接实体
//!
//! 对应 `short_links` 表，存储短码到目标 URL 的映射及点击计数。

use sea_orm::entity::prelude::*;

/// 一条短链接记录；`short_code` 即业务层的短码。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "short_links")]
pub struct Model {
    /// 主键，短码本身（非自增），全局唯一。
    #[sea_orm(primary_key, auto_increment = false)]
    pub short_code: String,
    /// 重定向目标 URL，完整形式（含 scheme）。
    #[sea_orm(column_type = "Text")]
    pub target_url: String,
    /// 创建时间（UTC）。
    pub created_at: DateTimeUtc,
    /// 过期时间（UTC）；NULL 表示永不过期。
    pub expires_at: Option<DateTimeUtc>,
    /// 访问口令的哈希值；NULL 表示链接无密码保护。
    pub password: Option<String>,
    /// 累计点击次数，默认 0。
    pub click_count: i64,
}

/// 独立实体，未声明与其他表的关联。
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
