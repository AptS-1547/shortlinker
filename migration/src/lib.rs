//! Shortlinker 数据库 migration crate。
//!
//! 按 `SeaORM` 约定提供版本化迁移和与表结构对应的实体定义；
//! 迁移必须保持幂等且按时间戳顺序执行，历史迁移的语义不可修改。
pub use sea_orm_migration::prelude::*;

/// `SeaORM` 实体定义，与各迁移创建出的表结构一一对应。
pub mod entities;
mod m020251023_000001_initial_table;
mod m020260111_000001_system_config;
mod m020260112_000001_search_index;
mod m20260202_000001_click_logs;
mod m20260206_000001_click_rollups;
mod m20260206_000002_analytics_indexes;
mod m20260207_000001_user_agents_table;
mod m20260207_000002_user_agents_parsed;
mod m20260208_000001_utm_source;
mod m20260208_000002_drop_user_agent;
mod m20260209_000001_source_geo_indexes;
mod m20260209_000002_analytics_indexes_v2;
mod m20260209_000003_global_daily_rollup;
mod m20260721_000001_forge_system_config;

/// 迁移入口，按声明顺序聚合全部迁移供 `sea-orm-cli` 与运行时使用。
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m020251023_000001_initial_table::Migration),
            Box::new(m020260111_000001_system_config::Migration),
            Box::new(m020260112_000001_search_index::Migration),
            Box::new(m20260202_000001_click_logs::Migration),
            Box::new(m20260206_000001_click_rollups::Migration),
            Box::new(m20260206_000002_analytics_indexes::Migration),
            Box::new(m20260207_000001_user_agents_table::Migration),
            Box::new(m20260207_000002_user_agents_parsed::Migration),
            Box::new(m20260208_000001_utm_source::Migration),
            Box::new(m20260208_000002_drop_user_agent::Migration),
            Box::new(m20260209_000001_source_geo_indexes::Migration),
            Box::new(m20260209_000002_analytics_indexes_v2::Migration),
            Box::new(m20260209_000003_global_daily_rollup::Migration),
            Box::new(m20260721_000001_forge_system_config::Migration),
        ]
    }
}
