use lifelog_core::*;
use lifelog_types::DataModality;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::db::CREATED_TABLES;

/// Schema definition for a single modality table.
pub(crate) struct TableSchema {
    /// Modality this schema applies to.
    modality: DataModality,
    /// DEFINE FIELD statements (use `{table}` as placeholder).
    fields_ddl: &'static str,
    /// DEFINE INDEX statements (use `{table}` as placeholder).
    indexes_ddl: &'static str,
}

/// Central schema registry for all data modalities.
static SCHEMAS: &[TableSchema] = &[
    TableSchema {
        modality: DataModality::Screen,
        fields_ddl: r#"
            DEFINE FIELD uuid        ON `{table}` TYPE string;
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD width       ON `{table}` TYPE int;
            DEFINE FIELD height      ON `{table}` TYPE int;
            DEFINE FIELD image_bytes ON `{table}` TYPE bytes;
            DEFINE FIELD mime_type   ON `{table}` TYPE string;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::Browser,
        fields_ddl: r#"
            DEFINE FIELD uuid        ON `{table}` TYPE string;
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD url         ON `{table}` TYPE string;
            DEFINE FIELD title       ON `{table}` TYPE string;
            DEFINE FIELD visit_count ON `{table}` TYPE int;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::Ocr,
        fields_ddl: r#"
            DEFINE FIELD uuid      ON `{table}` TYPE string;
            DEFINE FIELD timestamp ON `{table}` TYPE datetime;
            DEFINE FIELD text      ON `{table}` TYPE string;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::Audio,
        fields_ddl: r#"
            DEFINE FIELD uuid          ON `{table}` TYPE string;
            DEFINE FIELD timestamp     ON `{table}` TYPE datetime;
            DEFINE FIELD audio_bytes   ON `{table}` TYPE bytes;
            DEFINE FIELD codec         ON `{table}` TYPE string;
            DEFINE FIELD sample_rate   ON `{table}` TYPE int;
            DEFINE FIELD channels      ON `{table}` TYPE int;
            DEFINE FIELD duration_secs ON `{table}` TYPE float;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::Keystrokes,
        fields_ddl: r#"
            DEFINE FIELD uuid         ON `{table}` TYPE string;
            DEFINE FIELD timestamp    ON `{table}` TYPE datetime;
            DEFINE FIELD text         ON `{table}` TYPE string;
            DEFINE FIELD application  ON `{table}` TYPE string;
            DEFINE FIELD window_title ON `{table}` TYPE string;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::Clipboard,
        fields_ddl: r#"
            DEFINE FIELD uuid        ON `{table}` TYPE string;
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD text        ON `{table}` TYPE string;
            DEFINE FIELD binary_data ON `{table}` TYPE bytes;
            DEFINE FIELD mime_type   ON `{table}` TYPE string;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::ShellHistory,
        fields_ddl: r#"
            DEFINE FIELD uuid        ON `{table}` TYPE string;
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD command     ON `{table}` TYPE string;
            DEFINE FIELD working_dir ON `{table}` TYPE string;
            DEFINE FIELD exit_code   ON `{table}` TYPE int;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::WindowActivity,
        fields_ddl: r#"
            DEFINE FIELD uuid          ON `{table}` TYPE string;
            DEFINE FIELD timestamp     ON `{table}` TYPE datetime;
            DEFINE FIELD application   ON `{table}` TYPE string;
            DEFINE FIELD window_title  ON `{table}` TYPE string;
            DEFINE FIELD focused       ON `{table}` TYPE bool;
            DEFINE FIELD duration_secs ON `{table}` TYPE float;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::Mouse,
        fields_ddl: r#"
            DEFINE FIELD uuid           ON `{table}` TYPE string;
            DEFINE FIELD timestamp      ON `{table}` TYPE datetime;
            DEFINE FIELD activity_level ON `{table}` TYPE int;
            DEFINE FIELD button_mask    ON `{table}` TYPE int;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
];

/// Upload chunks metadata table schema.
static CHUNKS_DDL: &str = r#"
    DEFINE TABLE upload_chunks SCHEMALESS;
"#;

static WATERMARKS_DDL: &str = r#"
    DEFINE TABLE watermarks SCHEMAFULL;
    DEFINE FIELD last_timestamp ON watermarks TYPE datetime;
"#;

static CATALOG_DDL: &str = r#"
    DEFINE TABLE catalog SCHEMAFULL;
    DEFINE FIELD origin ON catalog TYPE string;
    DEFINE FIELD modality ON catalog TYPE string;
    DEFINE INDEX origin_idx ON catalog FIELDS origin, modality UNIQUE;
"#;

/// Look up the schema definition for a given modality.
pub(crate) fn schema_for(modality: DataModality) -> Option<&'static TableSchema> {
    SCHEMAS.iter().find(|s| s.modality == modality)
}

/// Ensure a modality table exists with full schema + indexes.
/// Idempotent: skips if the table was already created this process.
pub(crate) async fn ensure_table_schema(
    db: &Surreal<Client>,
    data_origin: &DataOrigin,
) -> surrealdb::Result<()> {
    let table = data_origin.get_table_name();
    if CREATED_TABLES.contains(&table) {
        return Ok(());
    }

    let modality = DataModality::from_str_name(&data_origin.modality_name).ok_or_else(|| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(format!(
            "Invalid modality name: {}",
            data_origin.modality_name
        )))
    })?;

    let schema = schema_for(modality).ok_or_else(|| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(format!(
            "No schema defined for modality {:?}",
            modality
        )))
    })?;

    let fields = schema.fields_ddl.replace("{table}", &table);
    let indexes = schema.indexes_ddl.replace("{table}", &table);

    let ddl = format!(
        r#"
        DEFINE TABLE `{table}` SCHEMAFULL;
        {fields}
        {indexes}
    "#
    );

    db.query(ddl.clone()).await?;
    CREATED_TABLES.insert(table.to_owned());
    tracing::info!(table = %table, "Ensured table schema");

    // Register in catalog
    let origin_str = match &data_origin.origin {
        DataOriginType::DeviceId(id) => id.clone(),
        DataOriginType::DataOrigin(o) => o.get_table_name(),
    };
    let modality_str = data_origin.modality_name.clone();

    let _ = db
        .query("UPSERT catalog SET origin = $origin, modality = $modality WHERE origin = $origin AND modality = $modality")
        .bind(("origin", origin_str))
        .bind(("modality", modality_str))
        .await;

    Ok(())
}

/// Ensure the upload_chunks metadata table exists.
pub(crate) async fn ensure_chunks_schema(db: &Surreal<Client>) -> surrealdb::Result<()> {
    if CREATED_TABLES.contains("upload_chunks") {
        return Ok(());
    }
    db.query(CHUNKS_DDL).await?.check()?;
    CREATED_TABLES.insert("upload_chunks".to_string());
    Ok(())
}

/// Ensure the watermarks table exists.
pub(crate) async fn ensure_watermarks_schema(db: &Surreal<Client>) -> surrealdb::Result<()> {
    if CREATED_TABLES.contains("watermarks") {
        return Ok(());
    }
    db.query(WATERMARKS_DDL).await?.check()?;
    CREATED_TABLES.insert("watermarks".to_string());
    Ok(())
}

/// Ensure the catalog table exists.
pub(crate) async fn ensure_catalog_schema(db: &Surreal<Client>) -> surrealdb::Result<()> {
    if CREATED_TABLES.contains("catalog") {
        return Ok(());
    }
    db.query(CATALOG_DDL).await?.check()?;
    CREATED_TABLES.insert("catalog".to_string());
    Ok(())
}

/// Run all schema migrations at startup.
/// Creates tables for every known origin already in the database
/// and ensures the chunks metadata table exists.
pub(crate) async fn run_startup_migrations(db: &Surreal<Client>) -> Result<(), LifelogError> {
    // Ensure upload_chunks table
    ensure_chunks_schema(db)
        .await
        .map_err(|e| LifelogError::Database(format!("chunks schema: {}", e)))?;

    // Ensure watermarks table
    ensure_watermarks_schema(db)
        .await
        .map_err(|e| LifelogError::Database(format!("watermarks schema: {}", e)))?;

    // Ensure catalog table
    ensure_catalog_schema(db)
        .await
        .map_err(|e| LifelogError::Database(format!("catalog schema: {}", e)))?;

    // Ensure tables for any existing origins
    let origins = crate::db::get_origins_from_db(db).await?;
    for origin in &origins {
        ensure_table_schema(db, origin)
            .await
            .map_err(|e| LifelogError::Database(format!("table schema for {}: {}", origin, e)))?;
    }

    Ok(())
}
