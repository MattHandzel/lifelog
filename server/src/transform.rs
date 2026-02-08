use data_modalities::*;
use lifelog_core::*;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::data_retrieval::get_data_by_key;
use crate::db::add_data_to_db;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum LifelogTransform {
    OcrTransform(OcrTransform),
}

impl LifelogTransform {
    pub fn id(&self) -> String {
        match self {
            LifelogTransform::OcrTransform(t) => format!("ocr-{}", t.source()),
        }
    }
    pub fn source(&self) -> DataOrigin {
        match self {
            LifelogTransform::OcrTransform(t) => t.source(),
        }
    }
}

impl From<OcrTransform> for LifelogTransform {
    fn from(transform: OcrTransform) -> Self {
        Self::OcrTransform(transform)
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
            Err(_) => continue,
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
                            let _ =
                                add_data_to_db::<OcrFrame, OcrFrame>(db, result, &t.destination())
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
