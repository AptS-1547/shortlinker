//! 小时级点击统计汇总实体

use sea_orm::entity::prelude::*;

/// 单条小时级汇总记录；`(short_code, hour_bucket)` 唯一。
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "click_stats_hourly")]
pub struct Model {
    /// 自增主键。
    #[sea_orm(primary_key)]
    pub id: i64,
    /// 该小时桶对应的短链接代码。
    pub short_code: String,
    /// 小时桶起始时间（UTC，分钟和秒截断为 00）。
    pub hour_bucket: DateTimeUtc,
    /// 该小时内该链接的点击总数，默认 0。
    pub click_count: i64,
    /// 该小时来源页面计数的 JSON 对象（referrer -> 次数）；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub referrer_counts: Option<String>,
    /// 该小时国家分布计数的 JSON 对象（国家代码 -> 次数）；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub country_counts: Option<String>,
    /// 该小时流量来源计数的 JSON 对象（`utm_source`/`ref:{domain}`/direct -> 次数）；NULL 表示无记录。
    #[sea_orm(column_type = "Text", nullable)]
    pub source_counts: Option<String>,
}

/// 独立汇总表，不与 `click_logs` 建立声明式关联。
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
