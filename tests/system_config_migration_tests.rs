use aster_forge_config::{ConfigSource, ConfigValueType, ConfigVisibility};
use aster_forge_db::system_config::{
    ActiveModel as ForgeSystemConfigActiveModel, Entity as ForgeSystemConfig, SystemConfigDbBinding,
};
use migration::entities::config_history;
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, EntityTrait, QueryFilter, Set,
};
use shortlinker::config::definitions::{CONFIG_REGISTRY, keys};

static SYSTEM_CONFIG_BINDING: SystemConfigDbBinding =
    SystemConfigDbBinding::new(&CONFIG_REGISTRY, &[]);

async fn migrate_to_legacy_schema(db: &sea_orm::DatabaseConnection) {
    let legacy_migration_count =
        u32::try_from(Migrator::migrations().len() - 1).expect("migration count should fit in u32");
    Migrator::up(db, Some(legacy_migration_count))
        .await
        .expect("legacy migrations should apply");
}

#[tokio::test]
async fn forge_system_config_migration_preserves_existing_rows_and_history() {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("SQLite should connect");
    migrate_to_legacy_schema(&db).await;

    db.execute_unprepared(
        r#"
        INSERT INTO system_config
            (key, value, value_type, requires_restart, is_sensitive, updated_at)
        VALUES
            ('analytics.sample_rate', '0.25', 'float', FALSE, FALSE, '2026-07-20T10:11:12Z'),
            ('api.jwt_secret', 'persisted-secret', 'string', TRUE, TRUE, '2026-07-20T10:12:13Z')
        "#,
    )
    .await
    .expect("legacy configuration rows should insert");
    db.execute_unprepared(
        r#"
        INSERT INTO config_history
            (config_key, old_value, new_value, changed_at, changed_by)
        VALUES
            ('api.jwt_secret', '[REDACTED]', '[REDACTED]', '2026-07-20T10:13:14Z', 'migration-test')
        "#,
    )
    .await
    .expect("legacy history row should insert");

    Migrator::up(&db, Some(1))
        .await
        .expect("Forge system_config migration should apply");

    let sample_rate = SYSTEM_CONFIG_BINDING
        .find_by_key(&db, keys::ANALYTICS_SAMPLE_RATE)
        .await
        .expect("sample rate query should succeed")
        .expect("sample rate should be preserved");
    assert_eq!(sample_rate.value, "0.25");
    assert_eq!(sample_rate.value_type, ConfigValueType::Number);
    assert!(!sample_rate.requires_restart);
    assert!(!sample_rate.is_sensitive);
    assert_eq!(sample_rate.source, ConfigSource::System);
    assert_eq!(sample_rate.visibility, ConfigVisibility::Private);
    assert!(sample_rate.namespace.is_empty());
    assert!(sample_rate.category.is_empty());
    assert!(sample_rate.description.is_empty());
    assert!(sample_rate.updated_by.is_none());

    let jwt_secret = SYSTEM_CONFIG_BINDING
        .find_by_key(&db, keys::API_JWT_SECRET)
        .await
        .expect("JWT secret query should succeed")
        .expect("JWT secret should be preserved");
    assert_eq!(jwt_secret.value, "persisted-secret");
    assert!(jwt_secret.requires_restart);
    assert!(jwt_secret.is_sensitive);

    let history = config_history::Entity::find()
        .filter(config_history::Column::ConfigKey.eq(keys::API_JWT_SECRET))
        .one(&db)
        .await
        .expect("history query should succeed")
        .expect("history row should be preserved");
    assert_eq!(history.old_value.as_deref(), Some("[REDACTED]"));
    assert_eq!(history.new_value, "[REDACTED]");
    assert_eq!(history.changed_by.as_deref(), Some("migration-test"));

    let duplicate = ForgeSystemConfigActiveModel {
        key: Set(keys::ANALYTICS_SAMPLE_RATE.to_string()),
        value: Set("0.5".to_string()),
        value_type: Set(ConfigValueType::Number),
        requires_restart: Set(false),
        is_sensitive: Set(false),
        source: Set(ConfigSource::System),
        visibility: Set(ConfigVisibility::Private),
        namespace: Set(String::new()),
        category: Set(String::new()),
        description: Set(String::new()),
        updated_at: Set(chrono::Utc::now()),
        updated_by: Set(None),
        ..Default::default()
    }
    .insert(&db)
    .await;
    assert!(duplicate.is_err(), "configuration keys must remain unique");

    SYSTEM_CONFIG_BINDING
        .ensure_defaults(&db)
        .await
        .expect("Forge defaults should seed and repair metadata");

    let repaired = SYSTEM_CONFIG_BINDING
        .find_by_key(&db, keys::ANALYTICS_SAMPLE_RATE)
        .await
        .expect("repaired sample rate query should succeed")
        .expect("sample rate should still exist");
    let definition = CONFIG_REGISTRY
        .get(keys::ANALYTICS_SAMPLE_RATE)
        .expect("sample rate definition should exist");
    assert_eq!(repaired.value, "0.25");
    assert_eq!(repaired.value_type, definition.value_type);
    assert_eq!(repaired.requires_restart, definition.requires_restart);
    assert_eq!(repaired.is_sensitive, definition.is_sensitive);
    assert_eq!(repaired.visibility, definition.visibility);
    assert_eq!(repaired.category, definition.category);
    assert_eq!(repaired.description, definition.description);
}

#[tokio::test]
async fn fresh_database_uses_forge_system_config_schema() {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("SQLite should connect");
    Migrator::up(&db, None)
        .await
        .expect("all migrations should apply to a fresh database");

    SYSTEM_CONFIG_BINDING
        .ensure_defaults(&db)
        .await
        .expect("defaults should seed on a fresh database");

    let rows = ForgeSystemConfig::find()
        .all(&db)
        .await
        .expect("Forge system config rows should query");
    assert_eq!(rows.len(), CONFIG_REGISTRY.definitions().len());
    assert!(rows.iter().all(|row| row.id > 0));
    assert!(rows.iter().all(|row| row.source == ConfigSource::System));
}
