pub mod click_log;
pub mod click_stats_daily;
pub mod click_stats_global_daily;
pub mod click_stats_global_hourly;
pub mod click_stats_hourly;
/// 配置变更历史实体（`config_history` 表）。
pub mod config_history;
/// 短链接实体（`short_links` 表）。
pub mod short_link;
pub mod user_agent;

pub use click_log::Entity as ClickLogEntity;
pub use click_stats_daily::Entity as ClickStatsDailyEntity;
pub use click_stats_global_daily::Entity as ClickStatsGlobalDailyEntity;
pub use click_stats_global_hourly::Entity as ClickStatsGlobalHourlyEntity;
pub use click_stats_hourly::Entity as ClickStatsHourlyEntity;
pub use config_history::Entity as ConfigHistoryEntity;
pub use short_link::Entity as ShortLinkEntity;
pub use user_agent::Entity as UserAgentEntity;
