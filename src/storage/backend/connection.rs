use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::info;

use crate::errors::{Result, ShortlinkerError};
use migration::{Migrator, MigratorTrait};

/// 连接 SQLite 数据库（带自动创建和性能优化）
pub async fn connect_sqlite(database_url: &str) -> Result<DatabaseConnection> {
    use sea_orm::SqlxSqliteConnector;
    use sea_orm::sqlx::SqlitePool;
    use sea_orm::sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
    use std::str::FromStr;

    let opt = SqliteConnectOptions::from_str(database_url)
        .map_err(|e| ShortlinkerError::database_config(format!("SQLite URL 解析失败: {}", e)))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .pragma("cache_size", "-64000")
        .pragma("temp_store", "memory")
        .pragma("mmap_size", "536870912")
        .pragma("wal_autocheckpoint", "1000");

    // 使用 sqlx 的连接池
    let pool = SqlitePool::connect_with(opt).await.map_err(|e| {
        ShortlinkerError::database_connection(format!("无法连接到 SQLite 数据库: {}", e))
    })?;

    // 转换为 Sea-ORM 的 DatabaseConnection
    Ok(SqlxSqliteConnector::from_sqlx_sqlite_pool(pool))
}

/// 连接通用数据库（MySQL/PostgreSQL）
pub async fn connect_generic(database_url: &str, backend_name: &str) -> Result<DatabaseConnection> {
    let config = crate::config::get_config();
    let pool_size = config.database.pool_size;

    let mut opt = ConnectOptions::new(database_url.to_owned());
    opt.max_connections(pool_size)
        .min_connections(pool_size.min(5))
        .connect_timeout(std::time::Duration::from_secs(8))
        .acquire_timeout(std::time::Duration::from_secs(8))
        .idle_timeout(std::time::Duration::from_secs(300)) // 5分钟空闲超时
        .max_lifetime(std::time::Duration::from_secs(3600)) // 1小时最大生命周期
        .sqlx_logging(false);

    Database::connect(opt).await.map_err(|e| {
        ShortlinkerError::database_connection(format!(
            "无法连接到 {} 数据库: {}",
            backend_name.to_uppercase(),
            e
        ))
    })
}

/// 运行数据库迁移
pub async fn run_migrations(db: &DatabaseConnection) -> Result<()> {
    Migrator::up(db, None)
        .await
        .map_err(|e| ShortlinkerError::database_operation(format!("迁移失败: {}", e)))?;

    info!("Database migrations completed");
    Ok(())
}
