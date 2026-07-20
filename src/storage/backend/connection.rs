use sea_orm::DatabaseConnection;
use tracing::info;

use crate::errors::{Result, ShortlinkerError};
use migration::{Migrator, MigratorTrait};

/// 运行数据库迁移
pub async fn run_migrations(db: &DatabaseConnection) -> Result<()> {
    Migrator::up(db, None)
        .await
        .map_err(|e| ShortlinkerError::database_operation(format!("Migration failed: {}", e)))?;

    info!("Database migrations completed");
    Ok(())
}
