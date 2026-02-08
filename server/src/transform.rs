use data_modalities::ocr::OcrTransform;
use lifelog_core::{DataOrigin, DateTime, LifelogFrameKey, LifelogImage, Transform, Utc};
use lifelog_types::{OcrFrame, ToRecord};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::data_retrieval::get_data_by_key;

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
    keys: &[LifelogFrameKey],
    transform: &LifelogTransform,
) -> Option<DateTime<Utc>> {
    let mut last_ts = None;

    for key in keys {
        let data_to_transform: lifelog_types::LifelogData = match get_data_by_key(db, key).await {
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
                        if let Ok(mut result) = t.apply(image) {
                            result.uuid = key.uuid.to_string();
                            let destination = t.destination();
                            let table = destination.get_table_name();
                            let id = result.uuid.clone();
                            let record = result.to_record();

                            // Ensure destination table exists
                            let _ = crate::schema::ensure_table_schema(db, &destination).await;

                            // Use native upsert to avoid serialization issues
                            let _ = db
                                .upsert::<Option<lifelog_types::OcrRecord>>((&table, &id))
                                .content(record)
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
