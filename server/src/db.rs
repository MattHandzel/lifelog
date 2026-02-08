use dashmap::DashSet;
use lifelog_core::DataType;
use lifelog_core::*;
use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use serde::Serialize;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

pub(crate) static CREATED_TABLES: Lazy<DashSet<String>> = Lazy::new(DashSet::new);

/// Reset the table cache — needed when restarting the server within the same process
/// (e.g., in integration tests that simulate server restarts).
#[cfg(test)]
pub fn reset_table_cache() {
    CREATED_TABLES.clear();
}

/// Ensure a table exists for the given data origin.
/// Delegates to the centralized schema registry.
pub(crate) async fn ensure_table(
    db: &Surreal<Client>,
    data_origin: &DataOrigin,
) -> surrealdb::Result<()> {
    crate::schema::ensure_table_schema(db, data_origin).await
}

pub(crate) async fn add_data_to_db<LifelogType, SurrealType>(
    db: &Surreal<Client>,
    data: LifelogType,
    data_origin: &DataOrigin,
) -> surrealdb::Result<SurrealType>
where
    LifelogType: Into<SurrealType> + DataType,
    SurrealType: Serialize + DeserializeOwned + 'static,
{
    let uuid = data.uuid();
    let table = data_origin.get_table_name();
    ensure_table(db, data_origin).await?;
    let data: SurrealType = data.into();
    let record: Option<SurrealType> = db
        .create((table.clone(), uuid.to_string()))
        .content(data)
        .await?;
    record.ok_or_else(|| {
        surrealdb::Error::Api(surrealdb::error::Api::Query(format!(
            "CREATE returned None for {table}:{uuid}"
        )))
    })
}

pub async fn get_tables(db: &Surreal<Client>) -> Result<Vec<String>, LifelogError> {
    #[derive(serde::Deserialize)]
    struct Info {
        tables: std::collections::HashMap<String, serde_json::Value>,
    }

    let mut resp = db
        .query("INFO FOR DB")
        .await
        .map_err(|e| LifelogError::Database(format!("{}", e)))?;

    let info: Option<Info> = resp
        .take(0)
        .map_err(|e| LifelogError::Database(format!("{}", e)))?;
    let info =
        info.ok_or_else(|| LifelogError::Database("INFO FOR DB returned no data".to_string()))?;
    Ok(info.tables.keys().cloned().collect())
}

pub(crate) async fn get_origins_from_db(
    db: &Surreal<Client>,
) -> Result<Vec<DataOrigin>, LifelogError> {
    let tables = get_tables(db).await?;
    let origins = tables
        .iter()
        .filter_map(|table| DataOrigin::tryfrom_string(table.clone()).ok())
        .collect();
    Ok(origins)
}

#[derive(Serialize, serde::Deserialize, Debug, Clone)]
pub struct Watermark {
    pub last_timestamp: DateTime<Utc>,
}

pub(crate) async fn get_watermark(
    db: &Surreal<Client>,
    id: &str,
) -> Result<DateTime<Utc>, LifelogError> {
    let w: Option<Watermark> = db
        .select(("watermarks", id))
        .await
        .map_err(|e| LifelogError::Database(e.to_string()))?;
    Ok(w.map(|w| w.last_timestamp)
        .unwrap_or_else(|| chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_default()))
}

pub(crate) async fn set_watermark(
    db: &Surreal<Client>,
    id: &str,
    ts: DateTime<Utc>,
) -> Result<(), LifelogError> {
    let _: Option<Watermark> = db
        .update(("watermarks", id))
        .content(Watermark { last_timestamp: ts })
        .await
        .map_err(|e| LifelogError::Database(e.to_string()))?;
    Ok(())
}
