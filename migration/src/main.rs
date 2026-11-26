//! migration 二进制入口，转发给 `SeaORM` CLI（`sea-orm-cli migrate ...`）。

use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
