pub use sea_orm_migration::prelude::*;

pub mod entities;
mod m020251023_000001_initial_table;
mod m020260111_000001_system_config;
mod m020260112_000001_search_index;
mod m20260202_000001_click_logs;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m020251023_000001_initial_table::Migration),
            Box::new(m020260111_000001_system_config::Migration),
            Box::new(m020260112_000001_search_index::Migration),
            Box::new(m20260202_000001_click_logs::Migration),
        ]
    }
}
