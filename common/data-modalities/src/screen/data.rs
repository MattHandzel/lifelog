use lifelog_core::*;

use lifelog_macros::lifelog_type;
use serde::{Deserialize, Serialize};

#[lifelog_type(Data)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenFrame {
    pub image_path: std::path::PathBuf,
    pub resolution: (u32, u32),
}

#[lifelog_type(Data)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetadata {
    pub dpi: f32,
    pub color_depth: u8,
    pub contains_sensitive: Option<bool>,
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
