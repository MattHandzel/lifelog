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
            DEFINE FIELD timestamp  ON `{table}` TYPE datetime;
            DEFINE FIELD width      ON `{table}` TYPE int;
            DEFINE FIELD height     ON `{table}` TYPE int;
            DEFINE FIELD imageBytes ON `{table}` TYPE string;
            DEFINE FIELD mimeType   ON `{table}` TYPE string;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::Browser,
        fields_ddl: r#"
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD url         ON `{table}` TYPE string;
            DEFINE FIELD title       ON `{table}` TYPE string;
            DEFINE FIELD visitCount  ON `{table}` TYPE int;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
            DEFINE INDEX `{table}_url_idx` ON `{table}` FIELDS url SEARCH ANALYZER SIMPLE BM25;
            DEFINE INDEX `{table}_title_idx` ON `{table}` FIELDS title SEARCH ANALYZER SIMPLE BM25;
        "#,
    },
    TableSchema {
        modality: DataModality::Ocr,
        fields_ddl: r#"
            DEFINE FIELD timestamp ON `{table}` TYPE datetime;
            DEFINE FIELD text      ON `{table}` TYPE string;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
            DEFINE INDEX `{table}_text_idx` ON `{table}` FIELDS text SEARCH ANALYZER SIMPLE BM25;
        "#,
    },
    TableSchema {
        modality: DataModality::Audio,
        fields_ddl: r#"
            DEFINE FIELD timestamp     ON `{table}` TYPE datetime;
            DEFINE FIELD audioBytes    ON `{table}` TYPE string;
            DEFINE FIELD codec         ON `{table}` TYPE string;
            DEFINE FIELD sampleRate    ON `{table}` TYPE int;
            DEFINE FIELD channels      ON `{table}` TYPE int;
            DEFINE FIELD durationSecs  ON `{table}` TYPE float;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
        "#,
    },
    TableSchema {
        modality: DataModality::Keystrokes,
        fields_ddl: r#"
            DEFINE FIELD timestamp    ON `{table}` TYPE datetime;
            DEFINE FIELD text         ON `{table}` TYPE string;
            DEFINE FIELD application  ON `{table}` TYPE string;
            DEFINE FIELD windowTitle  ON `{table}` TYPE string;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
            DEFINE INDEX `{table}_text_idx` ON `{table}` FIELDS text SEARCH ANALYZER SIMPLE BM25;
        "#,
    },
    TableSchema {
        modality: DataModality::Clipboard,
        fields_ddl: r#"
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD text        ON `{table}` TYPE string;
            DEFINE FIELD binaryData  ON `{table}` TYPE string;
            DEFINE FIELD mimeType    ON `{table}` TYPE string;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
            DEFINE INDEX `{table}_text_idx` ON `{table}` FIELDS text SEARCH ANALYZER SIMPLE BM25;
        "#,
    },
    TableSchema {
        modality: DataModality::ShellHistory,
        fields_ddl: r#"
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD command     ON `{table}` TYPE string;
            DEFINE FIELD workingDir  ON `{table}` TYPE string;
            DEFINE FIELD exitCode    ON `{table}` TYPE int;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
            DEFINE INDEX `{table}_cmd_idx` ON `{table}` FIELDS command SEARCH ANALYZER SIMPLE BM25;
        "#,
    },
    TableSchema {
        modality: DataModality::WindowActivity,
        fields_ddl: r#"
            DEFINE FIELD timestamp     ON `{table}` TYPE datetime;
            DEFINE FIELD application   ON `{table}` TYPE string;
            DEFINE FIELD windowTitle   ON `{table}` TYPE string;
            DEFINE FIELD focused       ON `{table}` TYPE bool;
            DEFINE FIELD durationSecs  ON `{table}` TYPE float;
        "#,
        indexes_ddl: r#"
            DEFINE INDEX `{table}_ts_idx` ON `{table}` FIELDS timestamp;
            DEFINE INDEX `{table}_app_idx` ON `{table}` FIELDS application SEARCH ANALYZER SIMPLE BM25;
            DEFINE INDEX `{table}_win_idx` ON `{table}` FIELDS windowTitle SEARCH ANALYZER SIMPLE BM25;
        "#,
    },
    TableSchema {
        modality: DataModality::Mouse,
        fields_ddl: r#"
            DEFINE FIELD timestamp      ON `{table}` TYPE datetime;
            DEFINE FIELD activityLevel  ON `{table}` TYPE int;
            DEFINE FIELD buttonMask     ON `{table}` TYPE int;
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

    // Ensure tables for any existing origins
    let origins = crate::db::get_origins_from_db(db).await?;
    for origin in &origins {
        ensure_table_schema(db, origin)
            .await
            .map_err(|e| LifelogError::Database(format!("table schema for {}: {}", origin, e)))?;
    }

    Ok(())
}
