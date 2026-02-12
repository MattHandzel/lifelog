use lifelog_core::uuid::Uuid;
use lifelog_core::*;
use lifelog_types::DataModality;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use utils::cas::FsCas;

fn time_quality_from_opt_str(s: Option<&str>) -> lifelog_types::TimeQuality {
    match s.unwrap_or_default() {
        "good" => lifelog_types::TimeQuality::Good,
        "degraded" => lifelog_types::TimeQuality::Degraded,
        _ => lifelog_types::TimeQuality::Unknown,
    }
}

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
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
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
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
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
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
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
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Interval as i32,
            };
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Audioframe(frame)),
            })
        }
        DataModality::Keystrokes => {
            let frame_record: lifelog_types::KeystrokeRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let frame = lifelog_types::KeystrokeFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                text: frame_record.text,
                application: frame_record.application,
                window_title: frame_record.window_title,
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
            };
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Keystrokeframe(frame)),
            })
        }
        DataModality::Clipboard => {
            let frame_record: lifelog_types::ClipboardRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let binary_data = if !frame_record.blob_hash.is_empty() {
                cas.get(&frame_record.blob_hash).map_err(|e| {
                    LifelogError::Database(format!(
                        "CAS read for {}: {}",
                        frame_record.blob_hash, e
                    ))
                })?
            } else {
                frame_record.binary_data
            };

            let frame = lifelog_types::ClipboardFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                text: frame_record.text,
                binary_data,
                mime_type: frame_record.mime_type,
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
            };
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Clipboardframe(frame)),
            })
        }
        DataModality::ShellHistory => {
            let frame_record: lifelog_types::ShellHistoryRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let frame = lifelog_types::ShellHistoryFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                command: frame_record.command,
                working_dir: frame_record.working_dir,
                exit_code: frame_record.exit_code,
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
            };
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Shellhistoryframe(
                    frame,
                )),
            })
        }
        DataModality::WindowActivity => {
            let frame_record: lifelog_types::WindowActivityRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let frame = lifelog_types::WindowActivityFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                application: frame_record.application,
                window_title: frame_record.window_title,
                focused: frame_record.focused,
                duration_secs: frame_record.duration_secs,
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Interval as i32,
            };
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Windowactivityframe(
                    frame,
                )),
            })
        }
        DataModality::Mouse => {
            let frame_record: lifelog_types::MouseRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let frame = lifelog_types::MouseFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                x: frame_record.x,
                y: frame_record.y,
                button: frame_record.button,
                pressed: frame_record.pressed,
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
            };
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Mouseframe(frame)),
            })
        }
        DataModality::Processes => {
            let frame_record: lifelog_types::ProcessRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let frame = lifelog_types::ProcessFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                processes: frame_record.processes,
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
            };
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
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
            };
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Cameraframe(frame)),
            })
        }
        DataModality::Weather => {
            let frame_record: lifelog_types::WeatherRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let frame = lifelog_types::WeatherFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                temperature: frame_record.temperature,
                humidity: frame_record.humidity,
                pressure: frame_record.pressure,
                conditions: frame_record.conditions,
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
            };
            Ok(lifelog_types::LifelogData {
                payload: Some(lifelog_types::lifelog_data::Payload::Weatherframe(frame)),
            })
        }
        DataModality::Hyprland => {
            let frame_record: lifelog_types::HyprlandRecord = db
                .select((&table, &*id))
                .await
                .map_err(|e| LifelogError::Database(format!("select {table}:{id}: {e}")))?
                .ok_or_else(|| LifelogError::Database(format!("record not found: {table}:{id}")))?;

            let frame = lifelog_types::HyprlandFrame {
                uuid: frame_record.uuid,
                timestamp: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                monitors: frame_record.monitors,
                workspaces: frame_record.workspaces,
                active_workspace: frame_record.active_workspace,
                clients: frame_record.clients,
                devices: frame_record.devices,
                cursor: frame_record.cursor,
                t_device: lifelog_types::to_pb_ts(frame_record.timestamp.0),
                t_ingest: frame_record
                    .t_ingest
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_canonical: frame_record
                    .t_canonical
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                t_end: frame_record
                    .t_end
                    .and_then(|t| lifelog_types::to_pb_ts(t.0)),
                time_quality: time_quality_from_opt_str(frame_record.time_quality.as_deref())
                    as i32,
                record_type: lifelog_types::RecordType::Point as i32,
            };
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
    let after_str = after.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    // In SurrealDB 2.x, if we want to ORDER BY a field, it must be part of the selection.
    let sql = format!("SELECT id, timestamp FROM `{table}` WHERE timestamp > d'{after_str}' ORDER BY timestamp ASC LIMIT {limit}");

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
