pub mod click_log;
pub mod click_stats_daily;
pub mod click_stats_global_daily;
pub mod click_stats_global_hourly;
pub mod click_stats_hourly;
pub mod config_history;
pub mod short_link;
pub mod system_config;
pub mod user_agent;

pub use click_log::Entity as ClickLogEntity;
pub use click_stats_daily::Entity as ClickStatsDailyEntity;
pub use click_stats_global_daily::Entity as ClickStatsGlobalDailyEntity;
pub use click_stats_global_hourly::Entity as ClickStatsGlobalHourlyEntity;
pub use click_stats_hourly::Entity as ClickStatsHourlyEntity;
pub use config_history::Entity as ConfigHistoryEntity;
pub use short_link::Entity as ShortLinkEntity;
pub use system_config::Entity as SystemConfigEntity;
pub use user_agent::Entity as UserAgentEntity;
