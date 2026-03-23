use std::sync::Arc;

use async_trait::async_trait;
use lifelog_core::time_skew::SkewEstimate;
use prost::Message;
use tokio::sync::RwLock;
use utils::ingest::IngestBackend;

use crate::postgres::PostgresPool;

pub struct UnifiedIngestBackend {
    pub pool: PostgresPool,
    pub cas: utils::cas::FsCas,
    pub skew_estimates: Arc<RwLock<std::collections::HashMap<String, SkewEstimate>>>,
    pub transforms: Vec<lifelog_types::TransformSpec>,
}

#[async_trait]
impl IngestBackend for UnifiedIngestBackend {
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
        let session_id_i64 = i64::try_from(session_id)
            .map_err(|_| format!("session_id overflow for postgres bigint: {session_id}"))?;
        let offset_i64 = i64::try_from(offset)
            .map_err(|_| format!("offset overflow for postgres bigint: {offset}"))?;
        let length_i32 = i32::try_from(length)
            .map_err(|_| format!("length overflow for postgres integer: {length}"))?;

        let lower_stream_id = stream_id.to_lowercase();
        let decode_result = decode_stream_payload(&lower_stream_id, payload);

        let (frame_uuid, indexed) = match decode_result {
            Some(Ok(data)) => {
                let mut row =
                    crate::frames::from_lifelog_data(collector_id, stream_id, &data, &self.cas)?;

                let (t_canonical, time_quality) =
                    get_canonical_time(&self.skew_estimates, collector_id, row.t_canonical).await;
                row.t_canonical = t_canonical;
                row.t_end = Some(
                    row.t_end
                        .map(|te| std::cmp::max(te, t_canonical))
                        .unwrap_or(t_canonical),
                );
                row.time_quality = time_quality;

                let is_screen = row.modality == "Screen";
                let ocr_enabled = is_screen
                    && self
                        .transforms
                        .iter()
                        .any(|spec| spec.enabled && spec.id.eq_ignore_ascii_case("ocr"));
                if ocr_enabled {
                    row.indexed = false;
                }

                let uuid_str = row.id.to_string();

                let client = self
                    .pool
                    .get()
                    .await
                    .map_err(|e| format!("postgres pool get failed: {e}"))?;

                client
                    .execute(crate::frames::FrameRow::insert_sql(), &row.insert_params())
                    .await
                    .map_err(|e| {
                        format!(
                            "postgres insert frames failed for {} (id={}): {e:?}",
                            row.modality, row.id
                        )
                    })?;

                let origin_key = format!("{}:{}", collector_id, row.modality);
                let collector_id_owned = collector_id.to_string();
                let stream_id_owned = stream_id.to_string();
                client
                    .execute(
                        "INSERT INTO catalog (origin, collector_id, modality, stream_id)
                         VALUES ($1, $2, $3, $4)
                         ON CONFLICT (origin) DO NOTHING",
                        &[
                            &origin_key,
                            &collector_id_owned,
                            &row.modality,
                            &stream_id_owned,
                        ],
                    )
                    .await
                    .map_err(|e| {
                        format!("postgres catalog registration failed for origin={origin_key}: {e}")
                    })?;

                (Some(uuid_str), true)
            }
            Some(Err(e)) => {
                tracing::warn!(
                    stream_id = %stream_id,
                    error = %e,
                    "Failed to decode frame from payload; treating as raw chunk for compatibility"
                );
                (None, true)
            }
            None => {
                tracing::debug!("No ingestion mapping for stream_id: {}", stream_id);
                (None, true)
            }
        };

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("postgres pool get failed: {e}"))?;

        let id_str = format!("{}_{}_{}_{}", collector_id, stream_id, session_id, offset);
        client
            .execute(
                "INSERT INTO upload_chunks (
                    id, collector_id, stream_id, session_id, \"offset\", length, hash, frame_uuid, indexed
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                ON CONFLICT (id) DO UPDATE
                SET indexed = (upload_chunks.indexed OR EXCLUDED.indexed)",
                &[
                    &id_str,
                    &collector_id,
                    &stream_id,
                    &session_id_i64,
                    &offset_i64,
                    &length_i32,
                    &hash,
                    &frame_uuid,
                    &indexed,
                ],
            )
            .await
            .map_err(|e| format!("postgres upsert upload_chunks failed: {e}"))?;

        Ok(())
    }

    async fn is_indexed(
        &self,
        collector_id: &str,
        stream_id: &str,
        session_id: u64,
        offset: u64,
    ) -> bool {
        let id = format!("{}_{}_{}_{}", collector_id, stream_id, session_id, offset);
        let client = match self.pool.get().await {
            Ok(c) => c,
            Err(_) => return false,
        };
        match client
            .query_opt("SELECT indexed FROM upload_chunks WHERE id = $1", &[&id])
            .await
        {
            Ok(Some(row)) => row.get::<_, bool>(0),
            Ok(None) => false,
            Err(_) => false,
        }
    }
}

fn decode_stream_payload(
    lower_stream_id: &str,
    payload: &[u8],
) -> Option<Result<lifelog_types::LifelogData, String>> {
    use lifelog_types::lifelog_data::Payload;

    let make_data = |p: Payload| lifelog_types::LifelogData { payload: Some(p) };

    match lower_stream_id {
        "screen" => Some(
            lifelog_types::ScreenFrame::decode(payload)
                .map(|f| make_data(Payload::Screenframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "browser" => Some(
            lifelog_types::BrowserFrame::decode(payload)
                .map(|f| make_data(Payload::Browserframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "audio" | "microphone" => Some(
            lifelog_types::AudioFrame::decode(payload)
                .map(|f| make_data(Payload::Audioframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "clipboard" => Some(
            lifelog_types::ClipboardFrame::decode(payload)
                .map(|f| make_data(Payload::Clipboardframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "shell_history" | "shellhistory" => Some(
            lifelog_types::ShellHistoryFrame::decode(payload)
                .map(|f| make_data(Payload::Shellhistoryframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "keystrokes" | "keyboard" => Some(
            lifelog_types::KeystrokeFrame::decode(payload)
                .map(|f| make_data(Payload::Keystrokeframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "mouse" => Some(
            lifelog_types::MouseFrame::decode(payload)
                .map(|f| make_data(Payload::Mouseframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "window_activity" | "windowactivity" => Some(
            lifelog_types::WindowActivityFrame::decode(payload)
                .map(|f| make_data(Payload::Windowactivityframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "process" | "processes" => Some(
            lifelog_types::ProcessFrame::decode(payload)
                .map(|f| make_data(Payload::Processframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "camera" => Some(
            lifelog_types::CameraFrame::decode(payload)
                .map(|f| make_data(Payload::Cameraframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "weather" => Some(
            lifelog_types::WeatherFrame::decode(payload)
                .map(|f| make_data(Payload::Weatherframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "hyprland" => Some(
            lifelog_types::HyprlandFrame::decode(payload)
                .map(|f| make_data(Payload::Hyprlandframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "ocr" => Some(
            lifelog_types::OcrFrame::decode(payload)
                .map(|f| make_data(Payload::Ocrframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "transcription" => Some(
            lifelog_types::TranscriptionFrame::decode(payload)
                .map(|f| make_data(Payload::Transcriptionframe(f)))
                .map_err(|e| e.to_string()),
        ),
        "embedding" => Some(
            lifelog_types::EmbeddingFrame::decode(payload)
                .map(|f| make_data(Payload::Embeddingframe(f)))
                .map_err(|e| e.to_string()),
        ),
        _ => None,
    }
}

async fn get_canonical_time(
    skew_estimates: &Arc<RwLock<std::collections::HashMap<String, SkewEstimate>>>,
    collector_id: &str,
    t_device: chrono::DateTime<chrono::Utc>,
) -> (chrono::DateTime<chrono::Utc>, String) {
    let skew_est = skew_estimates.read().await.get(collector_id).copied();
    match skew_est {
        Some(est) => (est.apply(t_device), est.time_quality.as_str().to_string()),
        None => (t_device, "unknown".to_string()),
    }
}
