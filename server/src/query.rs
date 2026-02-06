use crate::server::LifelogData;
use lifelog_core::uuid::Uuid;
use lifelog_types::*;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::surreal_types::*;

/// Validates a table name contains only safe characters (alphanumeric,
/// underscore, colon, hyphen). Prevents SQL injection via table names.
fn validate_table_name(name: String) -> Result<String, LifelogError> {
    if name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == ':' || c == '-')
    {
        Ok(name)
    } else {
        Err(LifelogError::Database(format!(
            "invalid table name: {name}"
        )))
    }
}

pub(crate) async fn get_all_uuids_from_origin(
    db: &Surreal<Client>,
    data_origin: &DataOrigin,
) -> Result<Vec<Uuid>, LifelogError> {
    let table = validate_table_name(data_origin.get_table_name())?;
    let sql = format!("SELECT VALUE record::id(id) as uuid FROM `{table}`");
    let uuids: Vec<String> = db
        .query(sql)
        .await
        .map_err(|e| LifelogError::Database(format!("query failed: {}", e)))?
        .take(0)
        .map_err(|e| LifelogError::Database(format!("take(0) failed: {}", e)))?;
    let uuids = uuids
        .into_iter()
        .filter_map(|s| s.parse::<Uuid>().ok())
        .collect();
    Ok(uuids)
}

pub(crate) async fn get_data_by_key(
    db: &Surreal<Client>,
    key: &LifelogFrameKey,
) -> Result<LifelogData, LifelogError> {
    let table = key.origin.get_table_name();
    let id = key.uuid.to_string();

    match key.origin.modality {
        DataModality::Screen => {
            let row: Option<ScreenFrameSurreal> = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?;
            let mut frame: data_modalities::ScreenFrame = row
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?
                .into();
            frame.uuid = key.uuid;
            Ok(frame.into())
        }
        DataModality::Ocr => {
            let row: Option<OcrFrameSurreal> = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?;
            let mut frame: data_modalities::OcrFrame = row
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?
                .into();
            frame.uuid = key.uuid;
            Ok(frame.into())
        }
        DataModality::Browser => {
            let row: Option<BrowserFrameSurreal> = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?;
            let mut frame: data_modalities::BrowserFrame = row
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?
                .into();
            frame.uuid = key.uuid;
            Ok(frame.into())
        }
    }
}
