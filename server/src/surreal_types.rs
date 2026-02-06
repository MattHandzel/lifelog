use data_modalities::*;
use lifelog_core::uuid;
use serde::{Deserialize, Serialize};

// TODO: Complete this for every data type, make this into a MACRO
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScreenFrameSurreal {
    //pub uuid: String,
    pub timestamp: surrealdb::sql::Datetime,
    pub width: i32,
    pub height: i32,
    pub image_bytes: surrealdb::sql::Bytes,
    pub mime_type: String,
}

// TDOO: Do this for every datatype
impl From<ScreenFrame> for ScreenFrameSurreal {
    fn from(frame: ScreenFrame) -> Self {
        Self {
            //uuid: frame.uuid.into(),
            timestamp: frame.timestamp.into(),
            width: frame.width as i32,
            height: frame.height as i32,
            image_bytes: frame.image_bytes.into(),
            mime_type: frame.mime_type,
        }
    }
}

impl From<ScreenFrameSurreal> for ScreenFrame {
    fn from(frame: ScreenFrameSurreal) -> Self {
        Self {
            uuid: uuid::Uuid::from_u128(0),
            timestamp: frame.timestamp.into(),
            width: frame.width as u32,
            height: frame.height as u32,
            image_bytes: frame.image_bytes.into(),
            mime_type: frame.mime_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrowserFrameSurreal {
    pub timestamp: surrealdb::sql::Datetime,
    pub url: String,
    pub title: String,
    pub visit_count: i32,
}

impl From<BrowserFrame> for BrowserFrameSurreal {
    fn from(frame: BrowserFrame) -> Self {
        Self {
            timestamp: frame.timestamp.into(),
            url: frame.url,
            title: frame.title,
            visit_count: frame.visit_count as i32,
        }
    }
}

impl From<BrowserFrameSurreal> for BrowserFrame {
    fn from(frame: BrowserFrameSurreal) -> Self {
        Self {
            uuid: uuid::Uuid::from_u128(0),
            timestamp: frame.timestamp.into(),
            url: frame.url,
            title: frame.title,
            visit_count: frame.visit_count as u32,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct OcrFrameSurreal {
    pub timestamp: surrealdb::sql::Datetime,
    pub text: String,
}

impl From<OcrFrame> for OcrFrameSurreal {
    fn from(frame: OcrFrame) -> Self {
        Self {
            timestamp: frame.timestamp.into(),
            text: frame.text,
        }
    }
}

impl From<OcrFrameSurreal> for OcrFrame {
    fn from(frame: OcrFrameSurreal) -> Self {
        Self {
            uuid: uuid::Uuid::from_u128(0),
            timestamp: frame.timestamp.into(),
            text: frame.text,
        }
    }
}
