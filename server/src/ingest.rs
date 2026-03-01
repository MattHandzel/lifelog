use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use config::load_transform_specs;
use lifelog_core::time_skew::SkewEstimate;
use lifelog_core::*;
use lifelog_types::{DataModality, ToRecord};
use prost::Message;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio::sync::RwLock;
use utils::ingest::IngestBackend;

use crate::schema::ensure_chunks_schema;
use crate::schema::ensure_table_schema;

pub(crate) struct ChunkRecord {
    pub(crate) collector_id: String,
    pub(crate) stream_id: String,
    pub(crate) session_id: u64,
    pub(crate) offset: u64,
    pub(crate) length: u32,
    pub(crate) hash: String,
    pub(crate) indexed: bool,
    pub(crate) frame_uuid: Option<String>,
}

pub struct SurrealIngestBackend {
    pub db: Surreal<Client>,
    pub cas: utils::cas::FsCas,
    pub skew_estimates: Arc<RwLock<std::collections::HashMap<String, SkewEstimate>>>,
}

impl SurrealIngestBackend {
    pub fn new(
        db: Surreal<Client>,
        cas: utils::cas::FsCas,
        skew_estimates: Arc<RwLock<std::collections::HashMap<String, SkewEstimate>>>,
    ) -> Self {
        Self {
            db,
            cas,
            skew_estimates,
        }
    }
}

#[async_trait]
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
        ensure_chunks_schema(&db)
            .await
            .map_err(|e: surrealdb::Error| e.to_string())?;

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
                } else {
                    tracing::warn!(
                        stream_id = %stream_id,
                        "Failed to decode frame from payload; treating as raw chunk for compatibility"
                    );
                    persisted_ok = true;
                    indexed = true;
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
                                    // If OCR transforms are enabled, keep ACK pinned until the
                                    // derived record is produced; otherwise screen is queryable now.
                                    let ocr_enabled = load_transform_specs().iter().any(|spec| {
                                        spec.enabled && spec.id.eq_ignore_ascii_case("ocr")
                                    });
                                    indexed = !ocr_enabled;
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
                } else {
                    tracing::warn!(
                        "Failed to decode ScreenFrame from payload; treating as raw chunk"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            "browser" => {
                ingest_frame!(
                    lifelog_types::BrowserFrame,
                    lifelog_types::BrowserRecord,
                    DataModality::Browser
                );
            }
            "mouse" => {
                ingest_frame!(
                    lifelog_types::MouseFrame,
                    lifelog_types::MouseRecord,
                    DataModality::Mouse
                );
            }
            "window_activity" | "windowactivity" => {
                if let Ok(frame) = lifelog_types::WindowActivityFrame::decode(payload) {
                    let origin = DataOrigin::new(
                        DataOriginType::DeviceId(collector_id.to_string()),
                        DataModality::WindowActivity.as_str_name().to_string(),
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
                            Some(est) => (
                                est.apply(t_device_dt),
                                est.time_quality.as_str().to_string(),
                            ),
                            None => (t_device_dt, "unknown".to_string()),
                        };

                        record.t_ingest = Some(now.clone());
                        record.t_canonical = Some(t_canonical_dt.into());
                        // Interval end from duration_secs; fallback to point semantics.
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
                            .upsert::<Option<lifelog_types::WindowActivityRecord>>((&table, &id))
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
                                    "WindowActivity frame ingestion failed"
                                );
                            }
                        }
                    }
                } else {
                    tracing::warn!(
                        "Failed to decode WindowActivityFrame from payload; treating as raw chunk"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            "keystrokes" | "keyboard" => {
                ingest_frame!(
                    lifelog_types::KeystrokeFrame,
                    lifelog_types::KeystrokeRecord,
                    DataModality::Keystrokes
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
                } else {
                    tracing::warn!(
                        "Failed to decode CameraFrame from payload; treating as raw chunk"
                    );
                    persisted_ok = true;
                    indexed = true;
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
                } else {
                    tracing::warn!(
                        "Failed to decode AudioFrame from payload; treating as raw chunk"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            "microphone" => {
                if let Ok(frame) = lifelog_types::AudioFrame::decode(payload) {
                    if frame.audio_bytes.is_empty() {
                        return Err("microphone frame has empty audio_bytes".to_string());
                    }
                    let blob_hash = match self.cas.put(&frame.audio_bytes) {
                        Ok(h) => h,
                        Err(e) => {
                            tracing::error!("CAS put failed for microphone blob: {}", e);
                            return Err(e.to_string());
                        }
                    };

                    let origin = DataOrigin::new(
                        DataOriginType::DeviceId(collector_id.to_string()),
                        DataModality::Microphone.as_str_name().to_string(),
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
                                    "Microphone frame ingestion failed"
                                );
                            }
                        }
                    }
                } else {
                    tracing::warn!("Failed to decode Microphone AudioFrame from payload; treating as raw chunk");
                    persisted_ok = true;
                    indexed = true;
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
                    let mut record = frame.to_record();
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
                    }

                    let origin = DataOrigin::new(
                        DataOriginType::DeviceId(collector_id.to_string()),
                        DataModality::Clipboard.as_str_name().to_string(),
                    );
                    if let Err(e) = ensure_table_schema(&db, &origin).await {
                        tracing::error!(
                            modality = "Clipboard",
                            error = %e,
                            "Failed to ensure table schema during Clipboard ingestion"
                        );
                    } else {
                        let table = origin.get_table_name();
                        let id = frame.uuid.clone();
                        frame_uuid = Some(id.clone());
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
                } else {
                    tracing::warn!(
                        "Failed to decode ClipboardFrame from payload; treating as raw chunk"
                    );
                    persisted_ok = true;
                    indexed = true;
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
                | "window_activity"
                | "windowactivity"
                | "keystrokes"
                | "keyboard"
                | "processes"
                | "camera"
                | "audio"
                | "microphone"
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

        // We use SET instead of CONTENT, and `indexed = (indexed = true OR $indexed = true)`
        // so that if an idempotent retry happens after `indexed` became true, we don't revert it to false.
        let q = "
            UPSERT type::thing('upload_chunks', $id)
            SET collector_id = $collector_id,
                stream_id = $stream_id,
                session_id = $session_id,
                offset = $offset,
                length = $length,
                hash = $hash,
                frame_uuid = $frame_uuid,
                indexed = (indexed = true OR $indexed = true)
        ";
        db.query(q)
            .bind(("id", id_str))
            .bind(("collector_id", collector_id.to_string()))
            .bind(("stream_id", stream_id.to_string()))
            .bind(("session_id", session_id))
            .bind(("offset", offset))
            .bind(("length", length))
            .bind(("hash", hash.to_string()))
            .bind(("frame_uuid", frame_uuid))
            .bind(("indexed", indexed))
            .await
            .map_err(|e: surrealdb::Error| e.to_string())?
            .check()
            .map_err(|e: surrealdb::Error| e.to_string())?;

        Ok(())
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
