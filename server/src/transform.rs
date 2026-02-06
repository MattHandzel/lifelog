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
        let data_to_transform: LifelogData = get_data_by_key(db, key)
            .await
            .expect(format!("Unable to get data by key: {}", key).as_str());
        println!(
            "[TRANSFORM_DATA]: Transforming data: <{}>:<{}>",
            key.origin.get_table_name(),
            key.uuid
        );
        for transform in transforms.iter() {
            // Check what transforms apply to these keys
            let transformed_data: Option<LifelogData> = match transform {
                LifelogTransform::OcrTransform(transform) => {
                    if key.origin == transform.source() {
                        let mut result = transform
                                .apply(data_to_transform.clone().try_into().expect("Data source is not a lifelog image!"))
                                .expect(format!("This should never error because the origins {} {} are the same", key.origin, transform.source()).as_str());

                        result.uuid = key.uuid; // NOTE: THIS IS IMPORTANT. THIS NEEDS TO BE FIXED
                                                // WITH A CODE REFACTOR
                        Some(result.into())
                    } else {
                        None
                    }
                }
            };
            let transformed_data = match transformed_data {
                Some(data) => data,
                None => continue,
            };
            match transformed_data {
                LifelogData::OcrFrame(ocr_frame) => {
                    add_data_to_db::<OcrFrame, OcrFrameSurreal>(
                        db,
                        ocr_frame,
                        &transform.destination(),
                    )
                    .await
                    .unwrap();
                }
                _ => unimplemented!(),
            }
        }
    }
}
