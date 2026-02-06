use data_modalities::*;
use lifelog_core::*;
use lifelog_proto::*;
use lifelog_proto::DataModality;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::db::add_data_to_db;
use crate::query::get_data_by_key;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum LifelogTransform {
    OcrTransform(OcrTransform),
}

impl LifelogTransform {
    pub fn source(&self) -> DataOrigin {
        match self {
            LifelogTransform::OcrTransform(transform) => transform.source(),
        }
    }
    pub fn destination(&self) -> DataOrigin {
        match self {
            LifelogTransform::OcrTransform(transform) => transform.destination(),
        }
    }
}

impl From<OcrTransform> for LifelogTransform {
    fn from(transform: OcrTransform) -> Self {
        Self::OcrTransform(transform)
    }
}

pub(crate) async fn transform_data(
    db: &Surreal<Client>,
    untransformed_data_keys: Vec<LifelogFrameKey>,
    transforms: Vec<LifelogTransform>,
) {
    for key in untransformed_data_keys.iter() {
        let data_to_transform: lifelog_proto::LifelogData = match get_data_by_key(db, key).await {
            Ok(data) => data,
            Err(_) => continue,
        };

        for transform in transforms.iter() {
            match transform {
                LifelogTransform::OcrTransform(transform) => {
                    if key.origin == transform.source() {
                        let payload = data_to_transform.payload.as_ref();
                        if let Some(lifelog_proto::lifelog_data::Payload::Screenframe(screen_frame)) = payload {
                            let image: LifelogImage = screen_frame.clone().into();
                            match transform.apply(image) {
                                Ok(mut result) => {
                                    result.uuid = key.uuid.to_string();
                                    let _ = add_data_to_db::<OcrFrame, OcrFrame>(
                                        db,
                                        result,
                                        &transform.destination(),
                                    )
                                    .await;
                                }
                                Err(_) => {}
                            }
                        }
                    }
                }
            };
        }
    }
}