use lifelog_core::{DataOrigin, DataOriginType};
use lifelog_types::DataModality;
use lifelog_types::ScreenFrame;
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

                if let Ok(_) = ensure_table_schema(&db, &origin).await {
                    let table = origin.get_table_name();
                    let id = frame.uuid.clone();

                    let json = match serde_json::to_string(&frame) {
                        Ok(s) => s,
                        Err(e) => {
                            tracing::error!(id = %id, error = %e, "Failed to serialize frame to JSON string");
                            return Err(e.to_string());
                        }
                    };

                    // Use a query with CONTENT string to bypass driver serialization issues.
                    let q = format!("CREATE `{table}`:`{id}` CONTENT {json}");
                    if let Ok(mut resp) = db.query(q).await {
                        let results: Vec<serde_json::Value> = resp.take(0).unwrap_or_default();
                        if !results.is_empty() {
                            indexed = true;
                        }
                    }
                }
            }
        }

        // Use a unique ID based on session and offset to ensure idempotency
        let id_str = format!("{}_{}_{}_{}", collector_id, stream_id, session_id, offset);

        let record = serde_json::json!({
            "collector_id": collector_id,
            "stream_id": stream_id,
            "session_id": session_id,
            "offset": offset,
            "length": length,
            "hash": hash,
            "indexed": indexed,
        });

        // Use a raw SQL query with CONTENT to avoid serialization issues
        let q = "UPSERT type::thing('upload_chunks', $id) CONTENT $record";
        let result = db
            .query(q)
            .bind(("id", id_str))
            .bind(("record", record))
            .await;

        match result {
            Ok(resp) => {
                let _ = resp.check();
                Ok(())
            }
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

        let id = format!("{}_{}_{}_{}", collector_id, stream_id, session_id, offset);

        let q =
            "SELECT VALUE indexed FROM upload_chunks WHERE id = type::thing('upload_chunks', $id)";
        match db.query(q).bind(("id", id)).await {
            Ok(mut resp) => {
                let results: Vec<bool> = resp.take(0).unwrap_or_default();
                results.first().cloned().unwrap_or(false)
            }
            Err(_) => false,
        }
    }
}
