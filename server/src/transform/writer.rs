use chrono::{DateTime, Utc};
use lifelog_core::{DataOrigin, LifelogError};
use lifelog_types::ToRecord;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::postgres::PostgresPool;

use super::TransformOutput;

pub struct SourceTimestamps {
    pub t_canonical: Option<::pbjson_types::Timestamp>,
    pub t_end: Option<::pbjson_types::Timestamp>,
    pub time_quality: String,
}

pub async fn write_transform_output(
    db: &Surreal<Client>,
    _postgres_pool: Option<&PostgresPool>,
    output: TransformOutput,
    destination: &DataOrigin,
    source_timestamps: &SourceTimestamps,
) -> Result<Option<DateTime<Utc>>, LifelogError> {
    let _ = crate::schema::ensure_table_schema(db, destination).await;
    let table = destination.get_table_name();

    match output {
        TransformOutput::Ocr(frame) => {
            let id = frame.uuid.clone();
            let mut record = frame.to_record();
            let now: surrealdb::sql::Datetime = chrono::Utc::now().into();
            record.t_ingest = Some(now);
            record.t_canonical = Some(lifelog_types::to_dt(source_timestamps.t_canonical).into());
            record.t_end = Some(lifelog_types::to_dt(source_timestamps.t_end).into());
            record.time_quality = Some(source_timestamps.time_quality.clone());

            let _ = db
                .upsert::<Option<lifelog_types::OcrRecord>>((&table, &id))
                .content(record)
                .await;

            let _ = db
                .query("UPDATE upload_chunks SET indexed = true WHERE frame_uuid = $uuid AND (stream_id = 'screen' OR stream_id = 'Screen')")
                .bind(("uuid", id))
                .await;

            extract_timestamp(source_timestamps.t_canonical)
        }
        TransformOutput::Transcription(frame) => {
            let id = frame.uuid.clone();
            let ts = frame.t_canonical.or(frame.timestamp);

            let record = lifelog_types::TranscriptionRecord {
                uuid: frame.uuid,
                timestamp: surrealdb::sql::Datetime::from(lifelog_types::to_dt(frame.timestamp)),
                text: frame.text,
                source_uuid: Some(frame.source_uuid),
                model: Some(frame.model),
                confidence: Some(frame.confidence),
                t_ingest: Some(surrealdb::sql::Datetime::from(chrono::Utc::now())),
                t_canonical: Some(surrealdb::sql::Datetime::from(lifelog_types::to_dt(ts))),
                t_end: Some(surrealdb::sql::Datetime::from(lifelog_types::to_dt(
                    frame.t_end.or(ts),
                ))),
                time_quality: Some(source_timestamps.time_quality.clone()),
            };

            let _ = db
                .upsert::<Option<lifelog_types::TranscriptionRecord>>((&table, &id))
                .content(record)
                .await;

            extract_timestamp(ts)
        }
        TransformOutput::Embedding(_frame) => {
            tracing::warn!("embedding output writing not yet implemented");
            Ok(None)
        }
    }
}

fn extract_timestamp(
    ts: Option<::pbjson_types::Timestamp>,
) -> Result<Option<DateTime<Utc>>, LifelogError> {
    Ok(ts.and_then(|t| DateTime::<Utc>::from_timestamp(t.seconds, t.nanos as u32)))
}
