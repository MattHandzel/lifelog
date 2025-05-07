use lifelog_core::*;

use lifelog_macros::lifelog_type;
use lifelog_proto;
use rand::distr::{Alphanumeric, Distribution, StandardUniform};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

#[lifelog_type(Data)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenFrame {
    pub width: u32,
    pub height: u32,
    pub image: Vec<u8>,
}

impl Modality for ScreenFrame {
    const TABLE: &'static str = "screen";
    fn into_payload(self) -> lifelog_proto::lifelog_data::Payload {
        lifelog_proto::lifelog_data::Payload::Screenframe(self.into()) // TODO: refactor code so this is
                                                                       // the same as screenframe
    }
    fn id(&self) -> String {
        self.uuid.to_string()
    }
}

impl Distribution<ScreenFrame> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ScreenFrame {
        let image_path: String = rng
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let width = rng.random_range(10..100); // TODO: REFACTOR THIS RANGE TO BE REALISTIC
        let height = rng.random_range(10..100); //TODO: REFACTOR THIS RANGE TO BE REALISTIC 
        let uuid = Uuid::new_v4(); // TODO: REfactor to use v6 (one version througohut the entire
                                   // project)
        let timestamp = Utc::now();
        let image: Vec<u8> = vec![0; (width * height) as usize]; // Placeholder for image data
        ScreenFrame {
            uuid,
            timestamp,
            width,
            height,
            image,
        }
    }
}

//impl crate::common::data_models::DataSchema for ScreenFrame {
//    fn table_name() -> &'static str {
//        "screen_frames"
//    }
//
//    fn schema() -> Vec<(&'static str, &'static str)> {
//        vec![
//            ("timestamp", "TIMESTAMP PRIMARY KEY"),
//            ("image_path", "TEXT NOT NULL"),
//            ("resolution_width", "INTEGER"),
//            ("resolution_height", "INTEGER"),
//            ("active_window", "TEXT"),
//            ("dpi", "REAL"),
//            ("color_depth", "SMALLINT"),
//        ]
//    }
//}
