mod db;
mod deserialize;
mod serialize;
#[cfg(test)]
mod tests;

pub use db::*;
pub use deserialize::*;
pub use serialize::*;

use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FrameRow {
    pub id: Uuid,
    pub collector_id: String,
    pub stream_id: String,
    pub modality: String,
    pub t_device: Option<DateTime<Utc>>,
    pub t_ingest: DateTime<Utc>,
    pub t_canonical: DateTime<Utc>,
    pub t_end: Option<DateTime<Utc>>,
    pub time_quality: String,
    pub blob_hash: Option<String>,
    pub blob_size: Option<i32>,
    pub indexed: bool,
    pub source_frame_id: Option<Uuid>,
    pub payload: JsonValue,
}

impl FrameRow {
    pub fn insert_sql() -> &'static str {
        "INSERT INTO frames (
            id, collector_id, stream_id, modality, time_range,
            t_device, t_ingest, t_canonical, t_end, time_quality,
            blob_hash, blob_size, indexed, source_frame_id, payload
        ) VALUES (
            $1, $2, $3, $4, tstzrange($5, $6, '[]'),
            $7, $8, $9, $10, $11,
            $12, $13, $14, $15, $16
        )
        ON CONFLICT (id) DO NOTHING"
    }

    pub fn insert_params(&self) -> [&(dyn tokio_postgres::types::ToSql + Sync); 16] {
        let range_start = &self.t_canonical;
        let range_end = &self.t_end;
        [
            &self.id,
            &self.collector_id,
            &self.stream_id,
            &self.modality,
            range_start,
            range_end.as_ref().unwrap_or(range_start),
            &self.t_device,
            &self.t_ingest,
            &self.t_canonical,
            &self.t_end,
            &self.time_quality,
            &self.blob_hash,
            &self.blob_size,
            &self.indexed,
            &self.source_frame_id,
            &self.payload,
        ]
    }
}
