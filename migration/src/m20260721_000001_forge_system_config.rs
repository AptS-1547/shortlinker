use aster_forge_config::{ConfigSource, ConfigValueType, ConfigVisibility};
use aster_forge_db::system_config::{
    ActiveModel as ForgeSystemConfigActiveModel, create_system_config_key_unique_index,
    create_system_config_table,
};
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ActiveModelTrait, EntityTrait, Set};

const UPDATED_AT_INDEX: &str = "idx_config_updated";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let legacy_rows = legacy_system_config::Entity::find().all(db).await?;

        manager
            .rename_table(
                Table::rename()
                    .table(SystemConfig::Table, LegacySystemConfigTable::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(create_system_config_table(db.get_database_backend()))
            .await?;

        for legacy in legacy_rows {
            ForgeSystemConfigActiveModel {
                key: Set(legacy.key),
                value: Set(legacy.value),
                value_type: Set(canonical_value_type(&legacy.value_type)),
                requires_restart: Set(legacy.requires_restart),
                is_sensitive: Set(legacy.is_sensitive),
                source: Set(ConfigSource::System),
                visibility: Set(ConfigVisibility::Private),
                namespace: Set(String::new()),
                category: Set(String::new()),
                description: Set(String::new()),
                updated_at: Set(legacy.updated_at),
                updated_by: Set(None),
                ..Default::default()
            }
            .insert(db)
            .await?;
        }

        manager
            .drop_table(
                Table::drop()
                    .table(LegacySystemConfigTable::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(create_system_config_key_unique_index())
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(UPDATED_AT_INDEX)
                    .table(SystemConfig::Table)
                    .col(SystemConfig::UpdatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let forge_rows = aster_forge_db::system_config::Entity::find()
            .all(db)
            .await?;

        manager
            .rename_table(
                Table::rename()
                    .table(SystemConfig::Table, ForgeSystemConfigTable::Table)
                    .to_owned(),
            )
            .await?;
        manager.create_table(create_legacy_table()).await?;

        for forge in forge_rows {
            legacy_system_config::ActiveModel {
                key: Set(forge.key),
                value: Set(forge.value),
                value_type: Set(forge.value_type.as_str().to_string()),
                requires_restart: Set(forge.requires_restart),
                is_sensitive: Set(forge.is_sensitive),
                updated_at: Set(forge.updated_at),
            }
            .insert(db)
            .await?;
        }

        manager
            .drop_table(
                Table::drop()
                    .table(ForgeSystemConfigTable::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name(UPDATED_AT_INDEX)
                    .table(SystemConfig::Table)
                    .col(SystemConfig::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
}

fn canonical_value_type(value: &str) -> ConfigValueType {
    match value {
        "multiline" => ConfigValueType::Multiline,
        "int" | "float" | "number" => ConfigValueType::Number,
        "bool" | "boolean" => ConfigValueType::Boolean,
        "enum" | "string_enum" => ConfigValueType::StringEnum,
        "stringarray" | "string_array" => ConfigValueType::StringArray,
        "enumarray" | "string_enum_set" => ConfigValueType::StringEnumSet,
        _ => ConfigValueType::String,
    }
}

fn create_legacy_table() -> TableCreateStatement {
    Table::create()
        .table(SystemConfig::Table)
        .col(
            ColumnDef::new(SystemConfig::Key)
                .string()
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new(SystemConfig::Value).text().not_null())
        .col(
            ColumnDef::new(SystemConfig::ValueType)
                .string()
                .not_null()
                .default("string"),
        )
        .col(
            ColumnDef::new(SystemConfig::RequiresRestart)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(SystemConfig::IsSensitive)
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(SystemConfig::UpdatedAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .to_owned()
}

#[derive(DeriveIden)]
enum SystemConfig {
    #[sea_orm(iden = "system_config")]
    Table,
    Key,
    Value,
    ValueType,
    RequiresRestart,
    IsSensitive,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum LegacySystemConfigTable {
    #[sea_orm(iden = "system_config_legacy")]
    Table,
}

#[derive(DeriveIden)]
enum ForgeSystemConfigTable {
    #[sea_orm(iden = "system_config_forge")]
    Table,
}

mod legacy_system_config {
    use sea_orm_migration::sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
    #[sea_orm(table_name = "system_config")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub key: String,
        #[sea_orm(column_type = "Text")]
        pub value: String,
        pub value_type: String,
        pub requires_restart: bool,
        pub is_sensitive: bool,
        pub updated_at: DateTimeUtc,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_value_types_are_canonicalized() {
        assert_eq!(canonical_value_type("int"), ConfigValueType::Number);
        assert_eq!(canonical_value_type("float"), ConfigValueType::Number);
        assert_eq!(canonical_value_type("bool"), ConfigValueType::Boolean);
        assert_eq!(
            canonical_value_type("stringarray"),
            ConfigValueType::StringArray
        );
        assert_eq!(
            canonical_value_type("enumarray"),
            ConfigValueType::StringEnumSet
        );
        assert_eq!(canonical_value_type("json"), ConfigValueType::String);
    }
}
