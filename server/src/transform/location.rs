use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::frames::FrameRow;
use crate::postgres::PostgresPool;
use lifelog_core::LifelogError;

/// Records location context based on collector_id heuristics.
/// Maps collector names to approximate locations (e.g., "laptop" → "mobile", "server" → "home").
pub async fn record_location_context(
    pool: &PostgresPool,
    collector_id: &str,
) -> Result<(), LifelogError> {
    let location = match collector_id {
        id if id.to_lowercase().contains("laptop") => "mobile",
        id if id.to_lowercase().contains("server") || id.to_lowercase().contains("desktop") => {
            "home"
        }
        _ => "unknown",
    };

    let now = Utc::now();
    let frame = FrameRow {
        id: Uuid::new_v4(),
        collector_id: collector_id.to_string(),
        stream_id: "location".to_string(),
        modality: "Location".to_string(),
        t_device: None,
        t_ingest: now,
        t_canonical: now,
        t_end: None,
        time_quality: "inferred".to_string(),
        blob_hash: None,
        blob_size: None,
        indexed: false,
        source_frame_id: None,
        payload: json!({
            "location": location,
            "source": "collector_id_heuristic"
        }),
    };

    crate::frames::upsert(pool, &frame).await
}
