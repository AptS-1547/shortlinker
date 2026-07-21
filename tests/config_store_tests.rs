use aster_forge_db::system_config::SystemConfigDbBinding;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectionTrait, Database};
use shortlinker::config::definitions::{CONFIG_REGISTRY, keys};
use shortlinker::storage::ConfigStore;

static SYSTEM_CONFIG_BINDING: SystemConfigDbBinding =
    SystemConfigDbBinding::new(&CONFIG_REGISTRY, &[]);

async fn config_store() -> (sea_orm::DatabaseConnection, ConfigStore) {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("SQLite should connect");
    Migrator::up(&db, None)
        .await
        .expect("migrations should apply");
    SYSTEM_CONFIG_BINDING
        .ensure_defaults(&db)
        .await
        .expect("configuration defaults should initialize");
    let store = ConfigStore::new(db.clone());
    (db, store)
}

#[tokio::test]
async fn config_store_records_redacted_history_only_for_real_changes() {
    let (_db, store) = config_store().await;

    let update = store
        .set(keys::API_JWT_SECRET, "replacement-secret")
        .await
        .expect("JWT secret should update");
    assert!(update.is_sensitive);
    assert_ne!(update.old_value.as_deref(), Some("replacement-secret"));

    let history = store
        .get_history(keys::API_JWT_SECRET, 10)
        .await
        .expect("JWT history should query");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].old_value.as_deref(), Some("[REDACTED]"));
    assert_eq!(history[0].new_value, "[REDACTED]");

    store
        .set(keys::API_JWT_SECRET, "replacement-secret")
        .await
        .expect("writing the same JWT secret should succeed");
    let unchanged_history = store
        .get_history(keys::API_JWT_SECRET, 10)
        .await
        .expect("unchanged JWT history should query");
    assert_eq!(unchanged_history.len(), 1);
}

#[tokio::test]
async fn config_store_rolls_back_value_when_history_write_fails() {
    let (db, store) = config_store().await;
    let before = SYSTEM_CONFIG_BINDING
        .find_by_key(&db, keys::ANALYTICS_SAMPLE_RATE)
        .await
        .expect("sample rate query should succeed")
        .expect("sample rate should exist");

    db.execute_unprepared("DROP TABLE config_history")
        .await
        .expect("history table should drop for rollback test");

    let update = store.set(keys::ANALYTICS_SAMPLE_RATE, "0.75").await;
    assert!(update.is_err(), "history failure should fail the update");

    let after = SYSTEM_CONFIG_BINDING
        .find_by_key(&db, keys::ANALYTICS_SAMPLE_RATE)
        .await
        .expect("sample rate query should succeed after rollback")
        .expect("sample rate should still exist");
    assert_eq!(after.value, before.value);
    assert_eq!(after.updated_at, before.updated_at);
}
