use lifelog_core::{DataOrigin, DataOriginType};
use lifelog_proto::DataModality;
use lifelog_proto::ScreenFrame;
use prost::Message;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use utils::ingest::IngestBackend;

use crate::schema::{ensure_chunks_schema, ensure_table_schema};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct ChunkRecord {
    pub collector_id: String,
    pub stream_id: String,
    pub session_id: u64,
    pub offset: u64,
    pub length: u64,
    pub hash: String,
    pub indexed: bool,
}

pub(crate) struct SurrealIngestBackend {
    pub db: Surreal<Client>,
}

#[async_trait::async_trait]
impl IngestBackend for SurrealIngestBackend {
    async fn persist_metadata(
        &self,
        collector_id: &str,
        stream_id: &str,
        session_id: u64,
        offset: u64,
        length: u64,
        hash: &str,
        payload: &[u8],
    ) -> Result<(), String> {
        let db = self.db.clone();
        ensure_chunks_schema(&db).await.map_err(|e| e.to_string())?;

        let mut indexed = false;

        // Try to index the content
        // TODO: Handle other modalities
        if stream_id.eq_ignore_ascii_case("screen") {
            if let Ok(frame) = ScreenFrame::decode(payload) {
                let origin = DataOrigin::new(
                    DataOriginType::DeviceId(collector_id.to_string()),
                    DataModality::Screen.as_str_name().to_string(),
                );

                #[allow(clippy::redundant_pattern_matching)]
                if let Ok(_) = ensure_table_schema(&db, &origin).await {
                    let table = origin.get_table_name();
                    // Insert into table. We use create with a UUID-based ID or let Surreal generate one?
                    // ScreenFrame has uuid field.
                    let id = &frame.uuid;
                    #[allow(clippy::redundant_pattern_matching)]
                    if let Ok(_) = db
                        .create::<Option<ScreenFrame>>((table, id))
                        .content(frame)
                        .await
                    {
                        indexed = true;
                    }
                }
            }
        }

        let record = ChunkRecord {
            collector_id: collector_id.to_string(),
            stream_id: stream_id.to_string(),
            session_id,
            offset,
            length,
            hash: hash.to_string(),
            indexed,
        };

        // Use a unique ID based on session and offset to ensure idempotency
        let id = format!("{}-{}-{}-{}", collector_id, stream_id, session_id, offset);
        tracing::debug!(id = %id, offset, length, indexed, "Persisting chunk metadata");

        // Use UPDATE/MERGE if it exists to update 'indexed' status?
        // Or overwrite?
        // If we processed it now and it was not indexed before, we want to update it.
        // If it was already indexed, we don't want to unset it (though here we just re-derived it).

        let result = db
            .update::<Option<ChunkRecord>>(("upload_chunks", &id))
            .content(record)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn is_indexed(
        &self,
        collector_id: &str,
        stream_id: &str,
        session_id: u64,
        offset: u64,
    ) -> bool {
        let db = self.db.clone();
        let _ = ensure_chunks_schema(&db).await;

        let id = format!("{}-{}-{}-{}", collector_id, stream_id, session_id, offset);
        let record: Option<ChunkRecord> = db.select(("upload_chunks", id)).await.unwrap_or(None);

        record.map(|r| r.indexed).unwrap_or(false)
    }
}
