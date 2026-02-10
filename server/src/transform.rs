use data_modalities::ocr::OcrTransform;
use lifelog_core::{DataOrigin, DateTime, LifelogFrameKey, LifelogImage, Transform, Utc};
use lifelog_types::ToRecord;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;
use utils::cas::FsCas;

use crate::data_retrieval::get_data_by_key;

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
}

impl From<OcrTransform> for LifelogTransform {
    fn from(t: OcrTransform) -> Self {
        LifelogTransform::OcrTransform(t)
    }
}

pub(crate) async fn transform_data_single(
    db: &Surreal<Client>,
    cas: &FsCas,
    keys: &[LifelogFrameKey],
    transform: &LifelogTransform,
) -> Option<DateTime<Utc>> {
    let mut last_ts = None;

    for key in keys {
        let data_to_transform: lifelog_types::LifelogData =
            match get_data_by_key(db, cas, key).await {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!(uuid = %key.uuid, error = %e, "Failed to get data by key");
                    continue;
                }
            };

        match transform {
            LifelogTransform::OcrTransform(t) => {
                if key.origin == t.source() {
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

                            // Ensure destination table exists
                            let _ = crate::schema::ensure_table_schema(db, &destination).await;

                            // Use native upsert to avoid serialization issues
                            let _ = db
                                .upsert::<Option<lifelog_types::OcrRecord>>((&table, &id))
                                .content(record)
                                .await;

                            // Spec §6.2.1: durable ACK implies queryable, including derived outputs.
                            // The ingest pipeline pins Screen chunk ACK until OCR for the same uuid exists.
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
