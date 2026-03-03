use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use lifelog_core::time_skew::SkewEstimate;
use lifelog_core::*;
use lifelog_types::{DataModality, ToRecord};
use pbjson_types::Timestamp as PbTimestamp;
use prost::Message;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use tokio::sync::RwLock;
use utils::ingest::IngestBackend;

use crate::postgres::PostgresPool;
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
    pub transforms: Vec<lifelog_types::TransformSpec>,
}

pub struct PostgresIngestBackend {
    pub pool: PostgresPool,
    pub cas: utils::cas::FsCas,
    pub skew_estimates: Arc<RwLock<std::collections::HashMap<String, SkewEstimate>>>,
    pub transforms: Vec<lifelog_types::TransformSpec>,
}

pub enum HybridIngestBackend {
    Surreal(SurrealIngestBackend),
    Postgres(PostgresIngestBackend),
}

#[async_trait]
impl IngestBackend for HybridIngestBackend {
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
        match self {
            Self::Surreal(inner) => {
                inner
                    .persist_metadata(
                        collector_id,
                        stream_id,
                        session_id,
                        offset,
                        length,
                        hash,
                        payload,
                    )
                    .await
            }
            Self::Postgres(inner) => {
                inner
                    .persist_metadata(
                        collector_id,
                        stream_id,
                        session_id,
                        offset,
                        length,
                        hash,
                        payload,
                    )
                    .await
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
        match self {
            Self::Surreal(inner) => {
                inner
                    .is_indexed(collector_id, stream_id, session_id, offset)
                    .await
            }
            Self::Postgres(inner) => {
                inner
                    .is_indexed(collector_id, stream_id, session_id, offset)
                    .await
            }
        }
    }
}

impl SurrealIngestBackend {
    pub fn new(
        db: Surreal<Client>,
        cas: utils::cas::FsCas,
        skew_estimates: Arc<RwLock<std::collections::HashMap<String, SkewEstimate>>>,
        transforms: Vec<lifelog_types::TransformSpec>,
    ) -> Self {
        Self {
            db,
            cas,
            skew_estimates,
            transforms,
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
                                    let ocr_enabled = self.transforms.iter().any(|spec| {
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

fn pb_to_dt(ts: Option<PbTimestamp>) -> chrono::DateTime<chrono::Utc> {
    let ts = ts.unwrap_or_default();
    chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap_or_else(|| {
        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
            chrono::NaiveDateTime::MIN,
            chrono::Utc,
        )
    })
}

fn opt_non_empty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
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

#[async_trait]
impl IngestBackend for PostgresIngestBackend {
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

        let mut persisted_ok = false;
        let mut frame_uuid: Option<String> = None;
        let mut indexed = false;
        let lower_stream_id = stream_id.to_lowercase();

        let client = self
            .pool
            .get()
            .await
            .map_err(|e| format!("postgres pool get failed: {e}"))?;
        let now = Utc::now();
        let now_s = now.to_rfc3339();

        match lower_stream_id.as_str() {
            "screen" => {
                if let Ok(frame) = lifelog_types::ScreenFrame::decode(payload) {
                    if frame.image_bytes.is_empty() {
                        return Err("screen frame has empty image_bytes".to_string());
                    }
                    let blob_hash = self
                        .cas
                        .put(&frame.image_bytes)
                        .map_err(|e| format!("CAS put failed for screen blob: {e}"))?;

                    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
                    let (t_canonical, time_quality) =
                        get_canonical_time(&self.skew_estimates, collector_id, t_device).await;
                    let t_end = t_canonical;

                    client
                        .execute(
                            "INSERT INTO screen_records (
                                id, collector_id, stream_id, time_range,
                                t_device, t_ingest, t_canonical, t_end, time_quality,
                                width, height, blob_hash, blob_size, mime_type
                            ) VALUES (
                                $1::uuid, $2, $3, tstzrange($4::timestamptz, $5::timestamptz, '[)'),
                                $6::timestamptz, $7::timestamptz, $8::timestamptz, $9::timestamptz, $10,
                                $11, $12, $13, $14, $15
                            )
                            ON CONFLICT (id) DO NOTHING",
                            &[
                                &frame.uuid,
                                &collector_id,
                                &stream_id,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &t_device.to_rfc3339(),
                                &now_s,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &time_quality,
                                &(frame.width as i64),
                                &(frame.height as i64),
                                &blob_hash,
                                &(frame.image_bytes.len() as i64),
                                &frame.mime_type,
                            ],
                        )
                        .await
                        .map_err(|e| format!("postgres insert screen_records failed: {e}"))?;
                    frame_uuid = Some(frame.uuid);
                    persisted_ok = true;
                    let ocr_enabled = self
                        .transforms
                        .iter()
                        .any(|spec| spec.enabled && spec.id.eq_ignore_ascii_case("ocr"));
                    indexed = !ocr_enabled;
                } else {
                    tracing::warn!(
                        stream_id = %stream_id,
                        "Failed to decode ScreenFrame from payload; treating as raw chunk for compatibility"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            "browser" => {
                if let Ok(frame) = lifelog_types::BrowserFrame::decode(payload) {
                    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
                    let (t_canonical, time_quality) =
                        get_canonical_time(&self.skew_estimates, collector_id, t_device).await;
                    let t_end = t_canonical;

                    client
                        .execute(
                            "INSERT INTO browser_records (
                                id, collector_id, stream_id, time_range,
                                t_device, t_ingest, t_canonical, t_end, time_quality,
                                url, title, visit_count
                            ) VALUES (
                                $1::uuid, $2, $3, tstzrange($4::timestamptz, $5::timestamptz, '[)'),
                                $6::timestamptz, $7::timestamptz, $8::timestamptz, $9::timestamptz, $10,
                                $11, $12, $13
                            )
                            ON CONFLICT (id) DO NOTHING",
                            &[
                                &frame.uuid,
                                &collector_id,
                                &stream_id,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &t_device.to_rfc3339(),
                                &now_s,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &time_quality,
                                &frame.url,
                                &frame.title,
                                &(frame.visit_count as i64),
                            ],
                        )
                        .await
                        .map_err(|e| format!("postgres insert browser_records failed: {e}"))?;
                    frame_uuid = Some(frame.uuid);
                    persisted_ok = true;
                    indexed = true;
                } else {
                    tracing::warn!(
                        stream_id = %stream_id,
                        "Failed to decode BrowserFrame from payload; treating as raw chunk for compatibility"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            "ocr" => {
                if let Ok(frame) = lifelog_types::OcrFrame::decode(payload) {
                    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
                    let (t_canonical, time_quality) =
                        get_canonical_time(&self.skew_estimates, collector_id, t_device).await;
                    let t_end = t_canonical;

                    client
                        .execute(
                            "INSERT INTO ocr_records (
                                id, collector_id, stream_id, source_frame_uuid, time_range,
                                t_device, t_ingest, t_canonical, t_end, time_quality, text
                            ) VALUES (
                                $1::uuid, $2, $3, NULL, tstzrange($4::timestamptz, $5::timestamptz, '[)'),
                                $6::timestamptz, $7::timestamptz, $8::timestamptz, $9::timestamptz, $10, $11
                            )
                            ON CONFLICT (id) DO NOTHING",
                            &[
                                &frame.uuid,
                                &collector_id,
                                &stream_id,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &t_device.to_rfc3339(),
                                &now_s,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &time_quality,
                                &frame.text,
                            ],
                        )
                        .await
                        .map_err(|e| format!("postgres insert ocr_records failed: {e}"))?;
                    frame_uuid = Some(frame.uuid);
                    persisted_ok = true;
                    indexed = true;
                } else {
                    tracing::warn!(
                        stream_id = %stream_id,
                        "Failed to decode OcrFrame from payload; treating as raw chunk for compatibility"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            "audio" | "microphone" => {
                if let Ok(frame) = lifelog_types::AudioFrame::decode(payload) {
                    if frame.audio_bytes.is_empty() {
                        return Err(format!("{stream_id} frame has empty audio_bytes"));
                    }
                    let blob_hash = self
                        .cas
                        .put(&frame.audio_bytes)
                        .map_err(|e| format!("CAS put failed for audio blob: {e}"))?;

                    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
                    let (t_canonical, time_quality) =
                        get_canonical_time(&self.skew_estimates, collector_id, t_device).await;
                    let t_end = t_canonical
                        + chrono::Duration::milliseconds(
                            (frame.duration_secs.max(0.0) as f64 * 1000.0).round() as i64,
                        );

                    client
                        .execute(
                            "INSERT INTO audio_records (
                                id, collector_id, stream_id, time_range,
                                t_device, t_ingest, t_canonical, t_end, time_quality,
                                blob_hash, blob_size, codec, sample_rate, channels, duration_secs
                            ) VALUES (
                                $1::uuid, $2, $3, tstzrange($4::timestamptz, $5::timestamptz, '[)'),
                                $6::timestamptz, $7::timestamptz, $8::timestamptz, $9::timestamptz, $10,
                                $11, $12, $13, $14, $15, $16
                            )
                            ON CONFLICT (id) DO NOTHING",
                            &[
                                &frame.uuid,
                                &collector_id,
                                &stream_id,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &t_device.to_rfc3339(),
                                &now_s,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &time_quality,
                                &blob_hash,
                                &(frame.audio_bytes.len() as i64),
                                &frame.codec,
                                &(frame.sample_rate as i64),
                                &(frame.channels as i64),
                                &(frame.duration_secs as f64),
                            ],
                        )
                        .await
                        .map_err(|e| format!("postgres insert audio_records failed: {e}"))?;
                    frame_uuid = Some(frame.uuid);
                    persisted_ok = true;
                    indexed = true;
                } else {
                    tracing::warn!(
                        stream_id = %stream_id,
                        "Failed to decode AudioFrame from payload; treating as raw chunk for compatibility"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            "clipboard" => {
                if let Ok(frame) = lifelog_types::ClipboardFrame::decode(payload) {
                    let (blob_hash, blob_size) = if frame.binary_data.is_empty() {
                        (None, None)
                    } else {
                        let h = self
                            .cas
                            .put(&frame.binary_data)
                            .map_err(|e| format!("CAS put failed for clipboard blob: {e}"))?;
                        (Some(h), Some(frame.binary_data.len() as i64))
                    };

                    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
                    let (t_canonical, time_quality) =
                        get_canonical_time(&self.skew_estimates, collector_id, t_device).await;
                    let t_end = t_canonical;
                    let text = opt_non_empty(&frame.text);
                    let mime_type = opt_non_empty(&frame.mime_type);

                    client
                        .execute(
                            "INSERT INTO clipboard_records (
                                id, collector_id, stream_id, time_range,
                                t_device, t_ingest, t_canonical, t_end, time_quality,
                                text, blob_hash, blob_size, mime_type
                            ) VALUES (
                                $1::uuid, $2, $3, tstzrange($4::timestamptz, $5::timestamptz, '[)'),
                                $6::timestamptz, $7::timestamptz, $8::timestamptz, $9::timestamptz, $10,
                                $11, $12, $13, $14
                            )
                            ON CONFLICT (id) DO NOTHING",
                            &[
                                &frame.uuid,
                                &collector_id,
                                &stream_id,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &t_device.to_rfc3339(),
                                &now_s,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &time_quality,
                                &text,
                                &blob_hash,
                                &blob_size,
                                &mime_type,
                            ],
                        )
                        .await
                        .map_err(|e| format!("postgres insert clipboard_records failed: {e}"))?;
                    frame_uuid = Some(frame.uuid);
                    persisted_ok = true;
                    indexed = true;
                } else {
                    tracing::warn!(
                        stream_id = %stream_id,
                        "Failed to decode ClipboardFrame from payload; treating as raw chunk for compatibility"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            "shell_history" | "shellhistory" => {
                if let Ok(frame) = lifelog_types::ShellHistoryFrame::decode(payload) {
                    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
                    let (t_canonical, time_quality) =
                        get_canonical_time(&self.skew_estimates, collector_id, t_device).await;
                    let t_end = t_canonical;
                    let working_dir = opt_non_empty(&frame.working_dir);
                    let exit_code = Some(frame.exit_code);

                    client
                        .execute(
                            "INSERT INTO shell_history_records (
                                id, collector_id, stream_id, time_range,
                                t_device, t_ingest, t_canonical, t_end, time_quality,
                                command, working_dir, exit_code
                            ) VALUES (
                                $1::uuid, $2, $3, tstzrange($4::timestamptz, $5::timestamptz, '[)'),
                                $6::timestamptz, $7::timestamptz, $8::timestamptz, $9::timestamptz, $10,
                                $11, $12, $13
                            )
                            ON CONFLICT (id) DO NOTHING",
                            &[
                                &frame.uuid,
                                &collector_id,
                                &stream_id,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &t_device.to_rfc3339(),
                                &now_s,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &time_quality,
                                &frame.command,
                                &working_dir,
                                &exit_code,
                            ],
                        )
                        .await
                        .map_err(|e| {
                            format!("postgres insert shell_history_records failed: {e}")
                        })?;
                    frame_uuid = Some(frame.uuid);
                    persisted_ok = true;
                    indexed = true;
                } else {
                    tracing::warn!(
                        stream_id = %stream_id,
                        "Failed to decode ShellHistoryFrame from payload; treating as raw chunk for compatibility"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            "keystrokes" | "keyboard" => {
                if let Ok(frame) = lifelog_types::KeystrokeFrame::decode(payload) {
                    let t_device = pb_to_dt(frame.t_device.clone().or(frame.timestamp.clone()));
                    let (t_canonical, time_quality) =
                        get_canonical_time(&self.skew_estimates, collector_id, t_device).await;
                    let t_end = t_canonical;
                    let application = opt_non_empty(&frame.application);
                    let window_title = opt_non_empty(&frame.window_title);

                    client
                        .execute(
                            "INSERT INTO keystroke_records (
                                id, collector_id, stream_id, time_range,
                                t_device, t_ingest, t_canonical, t_end, time_quality,
                                text, application, window_title
                            ) VALUES (
                                $1::uuid, $2, $3, tstzrange($4::timestamptz, $5::timestamptz, '[)'),
                                $6::timestamptz, $7::timestamptz, $8::timestamptz, $9::timestamptz, $10,
                                $11, $12, $13
                            )
                            ON CONFLICT (id) DO NOTHING",
                            &[
                                &frame.uuid,
                                &collector_id,
                                &stream_id,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &t_device.to_rfc3339(),
                                &now_s,
                                &t_canonical.to_rfc3339(),
                                &t_end.to_rfc3339(),
                                &time_quality,
                                &frame.text,
                                &application,
                                &window_title,
                            ],
                        )
                        .await
                        .map_err(|e| format!("postgres insert keystroke_records failed: {e}"))?;
                    frame_uuid = Some(frame.uuid);
                    persisted_ok = true;
                    indexed = true;
                } else {
                    tracing::warn!(
                        stream_id = %stream_id,
                        "Failed to decode KeystrokeFrame from payload; treating as raw chunk for compatibility"
                    );
                    persisted_ok = true;
                    indexed = true;
                }
            }
            _ => {
                tracing::debug!(
                    "No PostgreSQL ingestion mapping for stream_id: {}",
                    stream_id
                );
                indexed = true;
            }
        }

        let known_stream = matches!(
            lower_stream_id.as_str(),
            "screen"
                | "browser"
                | "ocr"
                | "audio"
                | "microphone"
                | "clipboard"
                | "shell_history"
                | "shellhistory"
                | "keystrokes"
                | "keyboard"
        );
        if known_stream && !persisted_ok {
            return Err(format!(
                "frame ingestion failed for stream '{stream_id}': metadata not persisted, ACK withheld"
            ));
        }

        let id_str = format!("{}_{}_{}_{}", collector_id, stream_id, session_id, offset);
        client
            .execute(
                "INSERT INTO upload_chunks (
                    id, collector_id, stream_id, session_id, offset, length, hash, frame_uuid, indexed
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
