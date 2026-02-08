use lifelog_core::uuid::Uuid;
use lifelog_core::*;
use lifelog_types::DataModality;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use utils::cas::FsCas;

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
    cas: &FsCas,
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
            let frame_record: lifelog_types::ScreenRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let image_bytes = cas.get(&frame_record.blob_hash).map_err(|e| {
                LifelogError::Database(format!("CAS read for {}: {}", frame_record.blob_hash, e))
            })?;

            let frame = lifelog_types::ScreenFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                width: frame_record.width,
                height: frame_record.height,
                image_bytes,
                mime_type: frame_record.mime_type,
            };

            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Screenframe(frame)),
            })
        }
        DataModality::Browser => {
            let frame_record: lifelog_types::BrowserRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let frame = lifelog_types::BrowserFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                url: frame_record.url,
                title: frame_record.title,
                visit_count: frame_record.visit_count,
            };

            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Browserframe(frame)),
            })
        }
        DataModality::Ocr => {
            let frame_record: lifelog_types::OcrRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let frame = lifelog_types::OcrFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                text: frame_record.text,
            };

            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Ocrframe(frame)),
            })
        }
        DataModality::Audio => {
            let frame_record: lifelog_types::AudioRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let audio_bytes = cas.get(&frame_record.blob_hash).map_err(|e| {
                LifelogError::Database(format!("CAS read for {}: {}", frame_record.blob_hash, e))
            })?;

            let frame = lifelog_types::AudioFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                audio_bytes,
                codec: frame_record.codec,
                sample_rate: frame_record.sample_rate,
                channels: frame_record.channels,
                duration_secs: frame_record.duration_secs,
            };
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
        DataModality::Processes => {
            let mut frame: lifelog_types::ProcessFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Processframe(frame)),
            })
        }
        DataModality::Camera => {
            let frame_record: lifelog_types::CameraRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let image_bytes = cas.get(&frame_record.blob_hash).map_err(|e| {
                LifelogError::Database(format!("CAS read for {}: {}", frame_record.blob_hash, e))
            })?;

            let frame = lifelog_types::CameraFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                width: frame_record.width,
                height: frame_record.height,
                image_bytes,
                mime_type: frame_record.mime_type,
                device: frame_record.device,
            };
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Cameraframe(frame)),
            })
        }
        DataModality::Weather => {
            let mut frame: lifelog_types::WeatherFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Weatherframe(frame)),
            })
        }
        DataModality::Hyprland => {
            let mut frame: lifelog_types::HyprlandFrame = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;
            frame.uuid = key.uuid.to_string();
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Hyprlandframe(frame)),
            })
        }
    }
}

#[allow(dead_code)]
#[derive(serde::Deserialize, Debug)]
struct KeyResult {
    id: surrealdb::sql::Thing,
    #[allow(dead_code)]
    timestamp: surrealdb::sql::Datetime,
}

pub(crate) async fn get_keys_after_timestamp(
    db: &Surreal<Client>,
    origin: &DataOrigin,
    after: DateTime<Utc>,
    limit: usize,
) -> Result<Vec<LifelogFrameKey>, LifelogError> {
    let table = validate_table_name(origin.get_table_name())?;
    let after_str = after.to_rfc3339();
    // In SurrealDB 2.x, if we want to ORDER BY a field, it must be part of the selection.
    let sql = format!("SELECT id, timestamp FROM `{table}` WHERE timestamp > '{after_str}' ORDER BY timestamp ASC LIMIT {limit}");

    let res: Vec<KeyResult> = db
        .query(sql)
        .await
        .map_err(|e| LifelogError::Database(format!("query failed: {}", e)))?
        .take(0)
        .map_err(|e| LifelogError::Database(format!("take(0) failed: {}", e)))?;

    let keys = res
        .into_iter()
        .filter_map(|v| {
            let id_str =
                v.id.id
                    .to_string()
                    .trim_matches('⟨')
                    .trim_matches('⟩')
                    .to_string();
            id_str
                .parse::<Uuid>()
                .ok()
                .map(|uuid| LifelogFrameKey::new(uuid, origin.clone()))
        })
        .collect();
    Ok(keys)
}
