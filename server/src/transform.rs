use crate::server::LifelogData;
use data_modalities::*;
use lifelog_types::*;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use crate::db::add_data_to_db;
use crate::query::get_data_by_key;
use crate::surreal_types::OcrFrameSurreal;

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
        let data_to_transform: LifelogData = match get_data_by_key(db, key).await {
            Ok(data) => data,
            Err(_) => continue,
        };

        for transform in transforms.iter() {
            let transformed_data: Option<LifelogData> = match transform {
                LifelogTransform::OcrTransform(transform) => {
                    if key.origin == transform.source() {
                        let image: LifelogImage = match data_to_transform.clone().try_into() {
                            Ok(img) => img,
                            Err(_) => continue,
                        };
                        match transform.apply(image) {
                            Ok(mut result) => {
                                result.uuid = key.uuid;
                                Some(result.into())
                            }
                            Err(_) => None,
                        }
                    } else {
                        None
                    }
                }
            };

            let Some(transformed_data) = transformed_data else {
                continue;
            };

            match transformed_data {
                LifelogData::OcrFrame(ocr_frame) => {
                    let _ = add_data_to_db::<OcrFrame, OcrFrameSurreal>(
                        db,
                        ocr_frame,
                        &transform.destination(),
                    )
                    .await;
                }
                _ => {}
            }
        }
    }
}
