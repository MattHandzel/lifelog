use dashmap::DashSet;
use lifelog_core::*;
use once_cell::sync::Lazy;
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
    let tables: Vec<String> = info.tables.keys().cloned().collect();
    Ok(tables)
}

pub(crate) async fn get_origins_from_db(
    db: &Surreal<Client>,
) -> Result<Vec<DataOrigin>, LifelogError> {
    #[derive(serde::Deserialize)]
    struct CatalogEntry {
        origin: String,
        modality: String,
    }

    let mut resp = db
        .query("SELECT origin, modality FROM catalog")
        .await
        .map_err(|e| LifelogError::Database(format!("{}", e)))?;

    let entries: Vec<CatalogEntry> = resp
        .take(0)
        .map_err(|e| LifelogError::Database(format!("{}", e)))?;

    let origins = entries
        .into_iter()
        .filter_map(|e| DataOrigin::tryfrom_string(format!("{}:{}", e.origin, e.modality)).ok())
        .collect();
    Ok(origins)
}

#[derive(Serialize, serde::Deserialize, Debug, Clone)]
pub struct Watermark {
    pub last_timestamp: surrealdb::sql::Datetime,
}

pub(crate) async fn get_watermark(
    db: &Surreal<Client>,
    id: &str,
) -> Result<DateTime<Utc>, LifelogError> {
    let w: Option<Watermark> = db
        .select(("watermarks", id))
        .await
        .map_err(|e| LifelogError::Database(e.to_string()))?;
    Ok(w.map(|w| w.last_timestamp.0)
        .unwrap_or_else(|| chrono::DateTime::<Utc>::from_timestamp(0, 0).unwrap_or_default()))
}

pub(crate) async fn set_watermark(
    db: &Surreal<Client>,
    id: &str,
    ts: DateTime<Utc>,
) -> Result<(), LifelogError> {
    let _: Option<Watermark> = db
        .upsert(("watermarks", id))
        .content(Watermark {
            last_timestamp: ts.into(),
        })
        .await
        .map_err(|e| LifelogError::Database(e.to_string()))?;
    Ok(())
}
