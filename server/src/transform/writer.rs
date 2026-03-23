use chrono::{DateTime, Utc};
use lifelog_core::{DataOrigin, LifelogError};

use crate::frames;
use crate::postgres::PostgresPool;

use super::TransformOutput;

pub struct SourceTimestamps {
    pub t_canonical: Option<::pbjson_types::Timestamp>,
    pub t_end: Option<::pbjson_types::Timestamp>,
    pub time_quality: String,
}

fn tq_str(q: i32) -> String {
    match lifelog_types::TimeQuality::try_from(q).unwrap_or(lifelog_types::TimeQuality::Unknown) {
        lifelog_types::TimeQuality::Good => "good",
        lifelog_types::TimeQuality::Degraded => "degraded",
        lifelog_types::TimeQuality::Unknown => "unknown",
    }
    .to_string()
}

pub fn extract_source_timestamps(data: &lifelog_types::LifelogData) -> SourceTimestamps {
    use lifelog_types::lifelog_data::Payload;
    let default = SourceTimestamps {
        t_canonical: None,
        t_end: None,
        time_quality: "unknown".to_string(),
    };
    let payload = match &data.payload {
        Some(p) => p,
        None => return default,
    };
    match payload {
        Payload::Screenframe(f) => SourceTimestamps {
            t_canonical: f.t_canonical.or(f.timestamp),
            t_end: f.t_end.or(f.t_canonical).or(f.timestamp),
            time_quality: tq_str(f.time_quality),
        },
        Payload::Audioframe(f) => SourceTimestamps {
            t_canonical: f.t_canonical.or(f.timestamp),
            t_end: f.t_end.or(f.t_canonical).or(f.timestamp),
            time_quality: tq_str(f.time_quality),
        },
        Payload::Ocrframe(f) => SourceTimestamps {
            t_canonical: f.t_canonical.or(f.timestamp),
            t_end: f.t_end.or(f.t_canonical).or(f.timestamp),
            time_quality: tq_str(f.time_quality),
        },
        Payload::Transcriptionframe(f) => SourceTimestamps {
            t_canonical: f.t_canonical.or(f.timestamp),
            t_end: f.t_end.or(f.t_canonical).or(f.timestamp),
            time_quality: tq_str(f.time_quality),
        },
        Payload::Browserframe(f) => SourceTimestamps {
            t_canonical: f.t_canonical.or(f.timestamp),
            t_end: f.t_end.or(f.t_canonical).or(f.timestamp),
            time_quality: tq_str(f.time_quality),
        },
        Payload::Keystrokeframe(f) => SourceTimestamps {
            t_canonical: f.t_canonical.or(f.timestamp),
            t_end: f.t_end.or(f.t_canonical).or(f.timestamp),
            time_quality: tq_str(f.time_quality),
        },
        _ => SourceTimestamps {
            t_canonical: None,
            t_end: None,
            time_quality: "unknown".to_string(),
        },
    }
}

fn pb_to_dt(ts: Option<pbjson_types::Timestamp>) -> DateTime<Utc> {
    let ts = ts.unwrap_or_default();
    DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap_or_else(|| {
        DateTime::<Utc>::from_naive_utc_and_offset(chrono::NaiveDateTime::MIN, Utc)
    })
}

pub async fn write_transform_output(
    pool: &PostgresPool,
    output: TransformOutput,
    destination: &DataOrigin,
    source_timestamps: &SourceTimestamps,
) -> Result<Option<DateTime<Utc>>, LifelogError> {
    let collector_id = match &destination.origin {
        lifelog_core::DataOriginType::DeviceId(id) => id.clone(),
        lifelog_core::DataOriginType::DataOrigin(parent) => match &parent.origin {
            lifelog_core::DataOriginType::DeviceId(id) => id.clone(),
            _ => "unknown".to_string(),
        },
    };
    let stream_id = destination.modality_name.to_lowercase();

    match output {
        TransformOutput::Ocr(frame) => {
            let t_canonical = pb_to_dt(source_timestamps.t_canonical);
            let t_end = pb_to_dt(source_timestamps.t_end);
            let source_uuid = uuid::Uuid::parse_str(&frame.uuid).ok();

            let mut row = frames::from_ocr(&collector_id, &stream_id, &frame, source_uuid);
            row.id = uuid::Uuid::new_v4();
            row.t_canonical = t_canonical;
            row.t_end = Some(t_end);
            row.time_quality = source_timestamps.time_quality.clone();
            row.t_ingest = Utc::now();

            frames::upsert(pool, &row).await?;

            extract_timestamp(source_timestamps.t_canonical)
        }
        TransformOutput::Transcription(frame) => {
            let ts = frame.t_canonical.or(frame.timestamp);

            let mut row = frames::from_transcription(&collector_id, &stream_id, &frame);
            row.id = uuid::Uuid::new_v4();
            row.t_canonical = pb_to_dt(ts);
            row.t_end = Some(pb_to_dt(frame.t_end.or(ts)));
            row.time_quality = source_timestamps.time_quality.clone();
            row.t_ingest = Utc::now();

            frames::upsert(pool, &row).await?;

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
