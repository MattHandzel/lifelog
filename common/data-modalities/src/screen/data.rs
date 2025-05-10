use lifelog_core::*;

use lifelog_macros::lifelog_type;
use lifelog_proto;
use lifelog_types::Modality;
use rand::distr::{Alphanumeric, Distribution, StandardUniform};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use image;
use image::io::Reader as ImageReader;
use std::io::Cursor;
#[lifelog_type(Data)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenFrame {
    pub width: u32,
    pub height: u32,
    pub image_bytes: Vec<u8>,
    pub mime_type: String, // TODO: Refactor this to use mime type object, not doing it rn because
                           // macro is a pain
}

impl From<ScreenFrame> for image::DynamicImage {
    fn from(frame: ScreenFrame) -> Self {
        ImageReader::new(Cursor::new(frame.image_bytes))
            .with_guessed_format()
            .expect("Unable to guess image format")
            .decode()
            .expect("Unable to decode image")
    }
}

impl Modality for ScreenFrame {
    fn into_payload(self) -> lifelog_proto::lifelog_data::Payload {
        lifelog_proto::lifelog_data::Payload::Screenframe(self.into()) // TODO: refactor code so this is
                                                                       // the same as screenframe
    }
    fn get_table_name() -> &'static str {
        "screen" // TODO: automatically generate this based on folder name
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
    DEFINE FIELD timestamp  ON `{table}` TYPE datetime;
    DEFINE FIELD width      ON `{table}` TYPE int;
    DEFINE FIELD height     ON `{table}` TYPE int;
    DEFINE FIELD image_bytes ON `{table}` TYPE bytes;
    DEFINE FIELD mime_type  ON `{table}` TYPE string;
"#
    }
}
impl Distribution<ScreenFrame> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ScreenFrame {
        let image_path: String = rng
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect();
        let width = rng.random_range(640..1920);
        let height = rng.random_range(480..1080);
        let uuid = Uuid::new_v4(); // TODO: REfactor to use v6 (one version througohut the entire
                                   // project)
        let timestamp = Utc::now();
        let image: Vec<u8> = vec![0; (width * height) as usize]; // Placeholder for image data
        ScreenFrame {
            uuid,
            timestamp,
            width,
            height,
            image_bytes: image,
            mime_type: "image/png".to_string(), // TODO: Refactor this to use mime type object, not doing it
                                                // rn because macro is a pain
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
