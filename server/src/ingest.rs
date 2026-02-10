use chrono::Utc;
use lifelog_core::{DataOrigin, DataOriginType};
use lifelog_types::DataModality;
use lifelog_types::ToRecord;
use prost::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    /// UUID of the decoded frame payload (when the stream is a known modality).
    #[serde(default)]
    pub frame_uuid: Option<String>,
    /// `true` means the chunk is fully queryable per Spec §6.2.1 (base record persisted and any
    /// required async work such as derived transforms completed).
    pub indexed: bool,
}

pub(crate) struct SurrealIngestBackend {
    pub db: Surreal<Client>,
    pub cas: utils::cas::FsCas,
    pub skew_estimates: Arc<
        tokio::sync::RwLock<
            std::collections::HashMap<String, lifelog_core::time_skew::SkewEstimate>,
        >,
    >,
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

        // `persisted_ok` means the typed record write succeeded (so the chunk isn't "lost").
        // `indexed` means "fully queryable" (Spec §6.2.1).
        let mut persisted_ok = false;
        let mut frame_uuid: Option<String> = None;
        let mut indexed = false;
        let lower_stream_id = stream_id.to_lowercase();

        // Helper macro to handle ingestion boilerplate
        macro_rules! ingest_frame {
            ($frame_type:ty, $record_type:ty, $modality:expr) => {{
                if let Ok(frame) = <$frame_type>::decode(payload) {
                    let origin = DataOrigin::new(
                        DataOriginType::DeviceId(collector_id.to_string()),
                        $modality.as_str_name().to_string(),
                    );

                    if ensure_table_schema(&db, &origin).await.is_ok() {
                        let table = origin.get_table_name();
                        let id = frame.uuid.clone();
                        frame_uuid = Some(id.clone());
                        let mut record = frame.to_record();
                        let now: surrealdb::sql::Datetime = Utc::now().into();

                        // Get skew estimate for this collector
                        let skew_est = self.skew_estimates.read().await.get(collector_id).copied();

                        // Apply skew estimate to t_canonical
                        let t_device_dt: chrono::DateTime<chrono::Utc> = record.timestamp.0;
                        let (t_canonical_dt, quality_str) = match skew_est {
                            Some(est) => (est.apply(t_device_dt), est.time_quality.as_str().to_string()),
                            None => (t_device_dt, "unknown".to_string()),
                        };

                        record.t_ingest = Some(now.clone());
                        record.t_canonical = Some(t_canonical_dt.into());
                        // Default interval end for point records.
                        record.t_end = Some(t_canonical_dt.into());
                        record.time_quality = Some(quality_str);

                        match db
                            .upsert::<Option<$record_type>>((&table, &id))
                            .content(record)
                            .await
                        {
                            Ok(result) => {
                                if result.is_some() {
                                    persisted_ok = true;
                                    // Most modalities are queryable as soon as the base record exists.
                                    // (Derived transforms are handled separately.)
                                    indexed = true;
                                } else {
                                    tracing::warn!(
                                        "Frame ingestion returned no results for {} in table {}",
                                        id,
                                        table
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    id = %id,
                                    error = %e,
                                    "Frame ingestion failed for table {}",
                                    table
                                );
                            }
                        }
                    }
                }
            }};
        }

        match lower_stream_id.as_str() {
            "screen" => {
                if let Ok(frame) = lifelog_types::ScreenFrame::decode(payload) {
                    if frame.image_bytes.is_empty() {
                        return Err("screen frame has empty image_bytes".to_string());
                    }
                    let blob_hash = match self.cas.put(&frame.image_bytes) {
                        Ok(h) => h,
                        Err(e) => {
                            tracing::error!("CAS put failed for screen blob: {}", e);
                            return Err(e.to_string());
                        }
                    };

                    let origin = DataOrigin::new(
                        DataOriginType::DeviceId(collector_id.to_string()),
                        DataModality::Screen.as_str_name().to_string(),
                    );
                    if ensure_table_schema(&db, &origin).await.is_ok() {
                        let table = origin.get_table_name();
                        let id = frame.uuid.clone();
                        frame_uuid = Some(id.clone());
                        let mut record = frame.to_record();
                        record.blob_hash = blob_hash;
                        record.blob_size = frame.image_bytes.len() as u64;
                        let now: surrealdb::sql::Datetime = Utc::now().into();

                        // Get skew estimate for this collector
                        let skew_est = self.skew_estimates.read().await.get(collector_id).copied();

                        // Apply skew estimate to t_canonical
                        let t_device_dt: chrono::DateTime<chrono::Utc> = record.timestamp.0;
                        let (t_canonical_dt, quality_str) = match skew_est {
                            Some(est) => (
                                est.apply(t_device_dt),
                                est.time_quality.as_str().to_string(),
                            ),
                            None => (t_device_dt, "unknown".to_string()),
                        };

                        record.t_ingest = Some(now.clone());
                        record.t_canonical = Some(t_canonical_dt.into());
                        // Default interval end for point records.
                        record.t_end = Some(t_canonical_dt.into());
                        record.time_quality = Some(quality_str);

                        match db
                            .upsert::<Option<lifelog_types::ScreenRecord>>((&table, &id))
                            .content(record)
                            .await
                        {
                            Ok(result) => {
                                if result.is_some() {
                                    persisted_ok = true;
                                    // Screen frames are not "fully queryable" until OCR (derived)
                                    // records have been produced for this uuid.
                                    indexed = false;
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    id = %id,
                                    error = %e,
                                    "Screen frame ingestion failed"
                                );
                            }
                        }
                    }
                }
            }
            "browser" => {
                ingest_frame!(
                    lifelog_types::BrowserFrame,
                    lifelog_types::BrowserRecord,
                    DataModality::Browser
                );
            }
            "processes" => {
                ingest_frame!(
                    lifelog_types::ProcessFrame,
                    lifelog_types::ProcessRecord,
                    DataModality::Processes
                );
            }
            "camera" => {
                if let Ok(frame) = lifelog_types::CameraFrame::decode(payload) {
                    if frame.image_bytes.is_empty() {
                        return Err("camera frame has empty image_bytes".to_string());
                    }
                    let blob_hash = match self.cas.put(&frame.image_bytes) {
                        Ok(h) => h,
                        Err(e) => {
                            tracing::error!("CAS put failed for camera blob: {}", e);
                            return Err(e.to_string());
                        }
                    };

                    let origin = DataOrigin::new(
                        DataOriginType::DeviceId(collector_id.to_string()),
                        DataModality::Camera.as_str_name().to_string(),
                    );
                    if ensure_table_schema(&db, &origin).await.is_ok() {
                        let table = origin.get_table_name();
                        let id = frame.uuid.clone();
                        frame_uuid = Some(id.clone());
                        let mut record = frame.to_record();
                        record.blob_hash = blob_hash;
                        record.blob_size = frame.image_bytes.len() as u64;
                        let now: surrealdb::sql::Datetime = Utc::now().into();

                        // Get skew estimate for this collector
                        let skew_est = self.skew_estimates.read().await.get(collector_id).copied();

                        // Apply skew estimate to t_canonical
                        let t_device_dt: chrono::DateTime<chrono::Utc> = record.timestamp.0;
                        let (t_canonical_dt, quality_str) = match skew_est {
                            Some(est) => (
                                est.apply(t_device_dt),
                                est.time_quality.as_str().to_string(),
                            ),
                            None => (t_device_dt, "unknown".to_string()),
                        };

                        record.t_ingest = Some(now.clone());
                        record.t_canonical = Some(t_canonical_dt.into());
                        // Default interval end for point records.
                        record.t_end = Some(t_canonical_dt.into());
                        record.time_quality = Some(quality_str);

                        match db
                            .upsert::<Option<lifelog_types::CameraRecord>>((&table, &id))
                            .content(record)
                            .await
                        {
                            Ok(result) => {
                                if result.is_some() {
                                    persisted_ok = true;
                                    indexed = true;
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    id = %id,
                                    error = %e,
                                    "Camera frame ingestion failed"
                                );
                            }
                        }
                    }
                }
            }
            "audio" => {
                if let Ok(frame) = lifelog_types::AudioFrame::decode(payload) {
                    if frame.audio_bytes.is_empty() {
                        return Err("audio frame has empty audio_bytes".to_string());
                    }
                    let blob_hash = match self.cas.put(&frame.audio_bytes) {
                        Ok(h) => h,
                        Err(e) => {
                            tracing::error!("CAS put failed for audio blob: {}", e);
                            return Err(e.to_string());
                        }
                    };

                    let origin = DataOrigin::new(
                        DataOriginType::DeviceId(collector_id.to_string()),
                        DataModality::Audio.as_str_name().to_string(),
                    );
                    if ensure_table_schema(&db, &origin).await.is_ok() {
                        let table = origin.get_table_name();
                        let id = frame.uuid.clone();
                        frame_uuid = Some(id.clone());
                        let mut record = frame.to_record();
                        record.blob_hash = blob_hash;
                        record.blob_size = frame.audio_bytes.len() as u64;
                        let now: surrealdb::sql::Datetime = Utc::now().into();

                        // Get skew estimate for this collector
                        let skew_est = self.skew_estimates.read().await.get(collector_id).copied();

                        // Apply skew estimate to t_canonical
                        let t_device_dt: chrono::DateTime<chrono::Utc> = record.timestamp.0;
                        let (t_canonical_dt, quality_str) = match skew_est {
                            Some(est) => (
                                est.apply(t_device_dt),
                                est.time_quality.as_str().to_string(),
                            ),
                            None => (t_device_dt, "unknown".to_string()),
                        };

                        record.t_ingest = Some(now.clone());
                        record.t_canonical = Some(t_canonical_dt.into());
                        // Audio is an interval record. Compute canonical end time from duration_secs.
                        let dur_ms =
                            if record.duration_secs.is_finite() && record.duration_secs > 0.0 {
                                (record.duration_secs as f64 * 1000.0).round() as i64
                            } else {
                                0
                            };
                        let t_end_dt = t_canonical_dt + chrono::Duration::milliseconds(dur_ms);
                        record.t_end = Some(t_end_dt.into());
                        record.time_quality = Some(quality_str);

                        match db
                            .upsert::<Option<lifelog_types::AudioRecord>>((&table, &id))
                            .content(record)
                            .await
                        {
                            Ok(result) => {
                                if result.is_some() {
                                    persisted_ok = true;
                                    indexed = true;
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    id = %id,
                                    error = %e,
                                    "Audio frame ingestion failed"
                                );
                            }
                        }
                    }
                }
            }
            "weather" => {
                ingest_frame!(
                    lifelog_types::WeatherFrame,
                    lifelog_types::WeatherRecord,
                    DataModality::Weather
                );
            }
            "hyprland" => {
                ingest_frame!(
                    lifelog_types::HyprlandFrame,
                    lifelog_types::HyprlandRecord,
                    DataModality::Hyprland
                );
            }
            "clipboard" => {
                if let Ok(frame) = lifelog_types::ClipboardFrame::decode(payload) {
                    let origin = DataOrigin::new(
                        DataOriginType::DeviceId(collector_id.to_string()),
                        DataModality::Clipboard.as_str_name().to_string(),
                    );

                    if ensure_table_schema(&db, &origin).await.is_ok() {
                        let table = origin.get_table_name();
                        let id = frame.uuid.clone();
                        frame_uuid = Some(id.clone());
                        let mut record = frame.to_record();
                        let now: surrealdb::sql::Datetime = Utc::now().into();

                        // If the clipboard includes a binary payload, store it in CAS and keep
                        // only a reference in SurrealDB. (Spec §6 / §8: blobs in CAS.)
                        if !frame.binary_data.is_empty() {
                            let blob_hash = match self.cas.put(&frame.binary_data) {
                                Ok(h) => h,
                                Err(e) => {
                                    tracing::error!("CAS put failed for clipboard blob: {}", e);
                                    return Err(e.to_string());
                                }
                            };
                            record.blob_hash = blob_hash;
                            record.blob_size = frame.binary_data.len() as u64;
                            record.binary_data.clear();
                        }

                        // Get skew estimate for this collector
                        let skew_est = self.skew_estimates.read().await.get(collector_id).copied();

                        // Apply skew estimate to t_canonical
                        let t_device_dt: chrono::DateTime<chrono::Utc> = record.timestamp.0;
                        let (t_canonical_dt, quality_str) = match skew_est {
                            Some(est) => (
                                est.apply(t_device_dt),
                                est.time_quality.as_str().to_string(),
                            ),
                            None => (t_device_dt, "unknown".to_string()),
                        };

                        record.t_ingest = Some(now.clone());
                        record.t_canonical = Some(t_canonical_dt.into());
                        // Default interval end for point records.
                        record.t_end = Some(t_canonical_dt.into());
                        record.time_quality = Some(quality_str);

                        match db
                            .upsert::<Option<lifelog_types::ClipboardRecord>>((&table, &id))
                            .content(record)
                            .await
                        {
                            Ok(result) => {
                                if result.is_some() {
                                    persisted_ok = true;
                                    indexed = true;
                                } else {
                                    tracing::warn!(
                                        "Frame ingestion returned no results for {} in table {}",
                                        id,
                                        table
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    id = %id,
                                    error = %e,
                                    "Clipboard frame ingestion failed for table {}",
                                    table
                                );
                            }
                        }
                    }
                }
            }
            "shell_history" | "shellhistory" => {
                ingest_frame!(
                    lifelog_types::ShellHistoryFrame,
                    lifelog_types::ShellHistoryRecord,
                    DataModality::ShellHistory
                );
            }
            _ => {
                tracing::debug!("No specific ingestion logic for stream_id: {}", stream_id);
                // Unknown stream: we only store raw chunks, so treat it as immediately queryable
                // (otherwise the ACK would never advance).
                indexed = true;
            }
        }

        // If we decoded a frame but failed to persist it, the collector must retry.
        // Only skip indexing check when the stream type has no ingestion logic
        // (unknown stream_id) — those are stored as raw chunks only.
        let known_stream = matches!(
            lower_stream_id.as_str(),
            "screen"
                | "browser"
                | "processes"
                | "camera"
                | "audio"
                | "weather"
                | "hyprland"
                | "clipboard"
                | "shell_history"
                | "shellhistory"
        );
        if known_stream && !persisted_ok {
            return Err(format!(
                "frame ingestion failed for stream '{}': metadata not persisted, ACK withheld",
                stream_id
            ));
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
            "frame_uuid": frame_uuid,
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
                resp.check().map_err(|e| e.to_string())?;
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
