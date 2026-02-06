use lifelog_types::*;
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
            DEFINE FIELD image_bytes ON `{table}` TYPE bytes;
            DEFINE FIELD mime_type  ON `{table}` TYPE string;
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
            DEFINE FIELD visit_count ON `{table}` TYPE int;
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
];

/// Upload chunks metadata table schema.
static CHUNKS_DDL: &str = r#"
    DEFINE TABLE upload_chunks SCHEMALESS;
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

    let schema = schema_for(data_origin.modality).ok_or_else(|| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(format!(
            "No schema defined for modality {:?}",
            data_origin.modality
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

/// Run all schema migrations at startup.
/// Creates tables for every known origin already in the database
/// and ensures the chunks metadata table exists.
pub(crate) async fn run_startup_migrations(db: &Surreal<Client>) -> Result<(), LifelogError> {
    // Ensure upload_chunks table
    ensure_chunks_schema(db)
        .await
        .map_err(|e| LifelogError::Database(format!("chunks schema: {}", e)))?;

    // Ensure tables for any existing origins
    let origins = crate::db::get_origins_from_db(db).await?;
    for origin in &origins {
        ensure_table_schema(db, origin)
            .await
            .map_err(|e| LifelogError::Database(format!("table schema for {}: {}", origin, e)))?;
    }

    Ok(())
}
