pub mod dag;
pub mod llm;
pub mod ocr;
pub mod stt;
pub mod watermark;
pub mod writer;

use async_trait::async_trait;
use lifelog_core::{DataOrigin, LifelogFrameKey};
use lifelog_types::LifelogData;

#[derive(Debug, thiserror::Error)]
pub enum TransformPipelineError {
    #[error("service unavailable: {endpoint}")]
    ServiceUnavailable { endpoint: String },
    #[error("service error: {0}")]
    ServiceError(String),
    #[error("data error: {0}")]
    DataError(String),
    #[error("unsupported input modality for transform {transform}: {modality}")]
    UnsupportedModality { transform: String, modality: String },
    #[error("cycle detected in transform DAG: {0}")]
    CycleDetected(String),
}

pub enum TransformOutput {
    Ocr(lifelog_types::OcrFrame),
    Transcription(lifelog_types::TranscriptionFrame),
    Embedding(lifelog_types::EmbeddingFrame),
}

#[async_trait]
pub trait TransformExecutor: Send + Sync {
    fn id(&self) -> &str;
    fn source_modality(&self) -> &str;
    fn destination_modality(&self) -> &str;
    fn priority(&self) -> u8;
    fn is_async(&self) -> bool;
    fn matches_origin(&self, key_origin: &DataOrigin) -> bool;
    fn source(&self) -> DataOrigin;
    fn destination(&self) -> DataOrigin;

    async fn execute(
        &self,
        http: &reqwest::Client,
        data: &LifelogData,
        key: &LifelogFrameKey,
    ) -> Result<TransformOutput, TransformPipelineError>;
}

// Re-export the legacy transform code for backward compatibility during migration.
// The old LifelogTransform enum and transform_data_single are preserved here
// so existing server.rs code continues to work while we wire up the new pipeline.

use data_modalities::ocr::OcrTransform;
use lifelog_core::{DataOriginType, DateTime, LifelogImage, Transform, Utc};
use lifelog_types::ToRecord;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use utils::cas::FsCas;

use crate::data_retrieval::get_data_by_key;
use crate::postgres::PostgresPool;

fn time_quality_str(q: i32) -> &'static str {
    match lifelog_types::TimeQuality::try_from(q).unwrap_or(lifelog_types::TimeQuality::Unknown) {
        lifelog_types::TimeQuality::Good => "good",
        lifelog_types::TimeQuality::Degraded => "degraded",
        lifelog_types::TimeQuality::Unknown => "unknown",
    }
}

#[derive(Debug, Clone)]
pub enum LifelogTransform {
    OcrTransform(OcrTransform),
}

impl LifelogTransform {
    pub fn id(&self) -> String {
        match self {
            LifelogTransform::OcrTransform(_) => "ocr".to_string(),
        }
    }

    pub fn source(&self) -> DataOrigin {
        match self {
            LifelogTransform::OcrTransform(t) => t.source(),
        }
    }

    #[allow(dead_code)]
    pub fn destination(&self) -> DataOrigin {
        match self {
            LifelogTransform::OcrTransform(t) => t.destination(),
        }
    }

    pub fn matches_origin(&self, key_origin: &DataOrigin) -> bool {
        match self {
            LifelogTransform::OcrTransform(t) => {
                let src = t.source();
                if src.modality_name != key_origin.modality_name {
                    return false;
                }

                match &src.origin {
                    DataOriginType::DataOrigin(o) if matches!(&**o, _) => src == *key_origin,
                    DataOriginType::DeviceId(device_id) if device_id == "*" => true,
                    _ => src == *key_origin,
                }
            }
        }
    }
}

impl From<OcrTransform> for LifelogTransform {
    fn from(t: OcrTransform) -> Self {
        LifelogTransform::OcrTransform(t)
    }
}

pub(crate) async fn transform_data_single(
    db: &Surreal<Client>,
    postgres_pool: Option<&PostgresPool>,
    cas: &FsCas,
    keys: &[LifelogFrameKey],
    transform: &LifelogTransform,
) -> Option<DateTime<Utc>> {
    let mut last_ts = None;

    for key in keys {
        let data_to_transform: lifelog_types::LifelogData =
            match get_data_by_key(db, postgres_pool, cas, key).await {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!(uuid = %key.uuid, error = %e, "Failed to get data by key");
                    continue;
                }
            };

        match transform {
            LifelogTransform::OcrTransform(t) => {
                if transform.matches_origin(&key.origin) {
                    let payload = data_to_transform.payload.as_ref();
                    if let Some(lifelog_types::lifelog_data::Payload::Screenframe(screen_frame)) =
                        payload
                    {
                        let image: LifelogImage = screen_frame.clone().into();
                        let src_t_canonical = screen_frame.t_canonical.or(screen_frame.timestamp);
                        let src_t_end = screen_frame
                            .t_end
                            .or(screen_frame.t_canonical)
                            .or(screen_frame.timestamp);
                        let src_quality = time_quality_str(screen_frame.time_quality).to_string();
                        if let Ok(mut result) = t.apply(image) {
                            result.uuid = key.uuid.to_string();
                            let destination = t.destination();
                            let table = destination.get_table_name();
                            let id = result.uuid.clone();
                            let mut record = result.to_record();
                            let now: surrealdb::sql::Datetime = chrono::Utc::now().into();
                            record.t_ingest = Some(now);
                            record.t_canonical = Some(lifelog_types::to_dt(src_t_canonical).into());
                            record.t_end = Some(lifelog_types::to_dt(src_t_end).into());
                            record.time_quality = Some(src_quality);

                            let _ = crate::schema::ensure_table_schema(db, &destination).await;

                            let _ = db
                                .upsert::<Option<lifelog_types::OcrRecord>>((&table, &id))
                                .content(record)
                                .await;

                            let _ = db
                                .query(
                                    "UPDATE upload_chunks SET indexed = true WHERE frame_uuid = $uuid AND (stream_id = 'screen' OR stream_id = 'Screen')",
                                )
                                .bind(("uuid", id.clone()))
                                .await;

                            if let Some(ts) = screen_frame.timestamp {
                                last_ts = Some(
                                    DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
                                        .unwrap_or_default(),
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    last_ts
}
