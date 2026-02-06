use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use utils::ingest::IngestBackend;

use crate::schema::ensure_chunks_schema;

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
    ) -> Result<(), String> {
        let db = self.db.clone();
        ensure_chunks_schema(&db).await.map_err(|e| e.to_string())?;

        let record = ChunkRecord {
            collector_id: collector_id.to_string(),
            stream_id: stream_id.to_string(),
            session_id,
            offset,
            length,
            hash: hash.to_string(),
            indexed: false,
        };

        // Use a unique ID based on session and offset to ensure idempotency
        let id = format!("{}-{}-{}-{}", collector_id, stream_id, session_id, offset);
        tracing::debug!(id = %id, offset, length, "Persisting chunk metadata");

        // Use CREATE to avoid overwriting existing records (preserving 'indexed' state)
        // If it exists, we assume it's the same chunk (idempotency)
        let result = db
            .create::<Option<ChunkRecord>>(("upload_chunks", &id))
            .content(record)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(surrealdb::Error::Db(surrealdb::error::Db::RecordExists { .. })) => Ok(()),
            Err(e) => {
                // Check if error string contains "already exists" as fallback for other error variants
                if e.to_string().contains("already exists") {
                    Ok(())
                } else {
                    Err(e.to_string())
                }
            }
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
