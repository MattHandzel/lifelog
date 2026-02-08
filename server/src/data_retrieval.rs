use lifelog_core::uuid::Uuid;
use lifelog_core::*;
use lifelog_types::DataModality;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

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

pub(crate) async fn get_data_by_key(
    db: &Surreal<Client>,
    key: &LifelogFrameKey,
) -> Result<lifelog_types::LifelogData, LifelogError> {
    let table = key.origin.get_table_name();
    let id = key.uuid.to_string();

    let modality = DataModality::from_str_name(&key.origin.modality_name).ok_or_else(|| {
        LifelogError::Database(format!(
            "Invalid modality name: {}",
            key.origin.modality_name
        ))
    })?;

    match modality {
        DataModality::Screen => {
            let mut frame: lifelog_types::ScreenFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Screenframe(frame)),
            })
        }
        DataModality::Ocr => {
            let mut frame: lifelog_types::OcrFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Ocrframe(frame)),
            })
        }
        DataModality::Browser => {
            let mut frame: lifelog_types::BrowserFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Browserframe(frame)),
            })
        }
        DataModality::Audio => {
            let mut frame: lifelog_types::AudioFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Audioframe(frame)),
            })
        }
        DataModality::Keystrokes => {
            let mut frame: lifelog_types::KeystrokeFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Keystrokeframe(frame)),
            })
        }
        DataModality::Clipboard => {
            let mut frame: lifelog_types::ClipboardFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Clipboardframe(frame)),
            })
        }
        DataModality::ShellHistory => {
            let mut frame: lifelog_types::ShellHistoryFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Shellhistoryframe(
                    frame,
                )),
            })
        }
        DataModality::WindowActivity => {
            let mut frame: lifelog_types::WindowActivityFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Windowactivityframe(
                    frame,
                )),
            })
        }
        DataModality::Mouse => {
            let mut frame: lifelog_types::MouseFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Mouseframe(frame)),
            })
        }
    }
}

#[allow(dead_code)]
pub(crate) async fn get_keys_after_timestamp(
    db: &Surreal<Client>,
    origin: &DataOrigin,
    after: DateTime<Utc>,
    limit: usize,
) -> Result<Vec<LifelogFrameKey>, LifelogError> {
    let table = validate_table_name(origin.get_table_name())?;
    let after_str = after.to_rfc3339();
    let sql = format!("SELECT VALUE record::id(id) as uuid FROM `{table}` WHERE timestamp > '{after_str}' ORDER BY timestamp ASC LIMIT {limit}");

    let uuids: Vec<String> = db
        .query(sql)
        .await
        .map_err(|e| LifelogError::Database(format!("query failed: {}", e)))?
        .take(0)
        .map_err(|e| LifelogError::Database(format!("take(0) failed: {}", e)))?;

    let uuids = uuids
        .into_iter()
        .filter_map(|s| s.parse::<Uuid>().ok())
        .map(|uuid| LifelogFrameKey::new(uuid, origin.clone()))
        .collect();
    Ok(uuids)
}
