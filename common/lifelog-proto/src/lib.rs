#![allow(clippy::needless_lifetimes, clippy::large_enum_variant)]

use image::ImageReader;
use lifelog_core::{
    DataOrigin, DataType, DateTime, LifelogError, LifelogFrameKey, LifelogImage, Modality,
    NaiveDateTime, Utc, Uuid as CoreUuid, Validate,
};
use rand::distr::{Distribution, StandardUniform};
use rand::Rng;
use std::io::Cursor;

pub mod lifelog {
    include!(concat!(env!("OUT_DIR"), "/lifelog.rs"));
    include!(concat!(env!("OUT_DIR"), "/lifelog.serde.rs"));
}

// Re-export lifelog members while avoiding name shadowing
pub use crate::lifelog::CollectorState;
pub use crate::lifelog::DataModality;
pub use crate::lifelog::SystemState;
pub use crate::lifelog::*;

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("lifelog_descriptor");

// --- Helper Functions ---

fn parse_uuid(s: &str) -> CoreUuid {
    s.parse().unwrap_or_else(|_| CoreUuid::nil())
}

fn to_dt(ts: Option<::pbjson_types::Timestamp>) -> DateTime<Utc> {
    let ts = ts.unwrap_or_default();
    DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
        .unwrap_or_else(|| DateTime::<Utc>::from_naive_utc_and_offset(NaiveDateTime::MIN, Utc))
}

pub fn to_pb_ts(dt: DateTime<Utc>) -> Option<::pbjson_types::Timestamp> {
    Some(::pbjson_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    })
}

// --- From implementations for Keys ---

impl TryFrom<LifelogDataKey> for LifelogFrameKey {
    type Error = LifelogError;

    fn try_from(key: LifelogDataKey) -> Result<Self, Self::Error> {
        Ok(LifelogFrameKey {
            uuid: parse_uuid(&key.uuid),
            origin: DataOrigin::tryfrom_string(key.origin).map_err(|e| {
                LifelogError::Validation {
                    field: "origin",
                    reason: format!("LifelogDataKey contained invalid origin: {e}"),
                }
            })?,
        })
    }
}

impl From<LifelogFrameKey> for LifelogDataKey {
    fn from(key: LifelogFrameKey) -> Self {
        LifelogDataKey {
            uuid: key.uuid.to_string(),
            origin: key.origin.get_table_name(),
        }
    }
}

// --- Trait Implementations ---

impl Validate for ServerConfig {
    fn validate(&self) -> Result<(), LifelogError> {
        if self.host.is_empty() {
            return Err(LifelogError::Validation {
                field: "host",
                reason: "must not be empty".to_string(),
            });
        }
        if self.port == 0 || self.port > 65535 {
            return Err(LifelogError::Validation {
                field: "port",
                reason: format!("must be between 1 and 65535, got {}", self.port),
            });
        }
        if self.database_endpoint.is_empty() {
            return Err(LifelogError::Validation {
                field: "database_endpoint",
                reason: "must not be empty".to_string(),
            });
        }
        if self.database_name.is_empty() {
            return Err(LifelogError::Validation {
                field: "database_name",
                reason: "must not be empty".to_string(),
            });
        }
        if self.server_name.is_empty() {
            return Err(LifelogError::Validation {
                field: "server_name",
                reason: "must not be empty".to_string(),
            });
        }
        Ok(())
    }
}

impl Validate for CollectorConfig {
    fn validate(&self) -> Result<(), LifelogError> {
        if self.id.is_empty() {
            return Err(LifelogError::Validation {
                field: "id",
                reason: "collector ID must not be empty".to_string(),
            });
        }
        if self.host.is_empty() {
            return Err(LifelogError::Validation {
                field: "host",
                reason: "must not be empty".to_string(),
            });
        }
        if self.port == 0 || self.port > 65535 {
            return Err(LifelogError::Validation {
                field: "port",
                reason: format!("must be between 1 and 65535, got {}", self.port),
            });
        }
        Ok(())
    }
}

// ScreenFrame
impl DataType for ScreenFrame {
    fn uuid(&self) -> CoreUuid {
        parse_uuid(&self.uuid)
    }
    fn timestamp(&self) -> DateTime<Utc> {
        to_dt(self.timestamp)
    }
}
impl Modality for ScreenFrame {
    fn get_table_name() -> &'static str {
        "screen"
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

impl From<ScreenFrame> for LifelogImage {
    fn from(frame: ScreenFrame) -> Self {
        let uuid = frame.uuid();
        let timestamp = frame.timestamp();
        LifelogImage {
            uuid,
            timestamp,
            image: image::DynamicImage::from(frame),
        }
    }
}

impl From<ScreenFrame> for image::DynamicImage {
    fn from(frame: ScreenFrame) -> Self {
        let reader = match ImageReader::new(Cursor::new(frame.image_bytes)).with_guessed_format() {
            Ok(r) => r,
            Err(e) => {
                ::tracing::warn!(uuid = %frame.uuid, error = %e, "Unable to guess image format, returning fallback image");
                return image::DynamicImage::new_rgba8(1, 1);
            }
        };
        match reader.decode() {
            Ok(img) => img,
            Err(e) => {
                ::tracing::warn!(uuid = %frame.uuid, error = %e, "Unable to decode image, returning fallback image");
                image::DynamicImage::new_rgba8(1, 1)
            }
        }
    }
}

impl Distribution<ScreenFrame> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ScreenFrame {
        let width = rng.random_range(640..1920);
        let height = rng.random_range(480..1080);
        let uuid = CoreUuid::new_v4().to_string();
        let timestamp = Utc::now();
        let image: Vec<u8> = vec![0; (width * height) as usize];
        ScreenFrame {
            uuid,
            timestamp: Some(::pbjson_types::Timestamp {
                seconds: timestamp.timestamp(),
                nanos: timestamp.timestamp_subsec_nanos() as i32,
            }),
            width,
            height,
            image_bytes: image,
            mime_type: "image/png".to_string(),
        }
    }
}

// BrowserFrame
impl DataType for BrowserFrame {
    fn uuid(&self) -> CoreUuid {
        parse_uuid(&self.uuid)
    }
    fn timestamp(&self) -> DateTime<Utc> {
        to_dt(self.timestamp)
    }
}
impl Modality for BrowserFrame {
    fn get_table_name() -> &'static str {
        "browser"
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD url         ON `{table}` TYPE string;
            DEFINE FIELD title       ON `{table}` TYPE string;
            DEFINE FIELD visit_count ON `{table}` TYPE int;
        "#
    }
}

// OcrFrame
impl DataType for OcrFrame {
    fn uuid(&self) -> CoreUuid {
        parse_uuid(&self.uuid)
    }
    fn timestamp(&self) -> DateTime<Utc> {
        to_dt(self.timestamp)
    }
}
impl Modality for OcrFrame {
    fn get_table_name() -> &'static str {
        "ocr"
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
            DEFINE FIELD timestamp ON `{table}` TYPE datetime;
            DEFINE FIELD text      ON `{table}` TYPE string;
        "#
    }
}

// AudioFrame
impl DataType for AudioFrame {
    fn uuid(&self) -> CoreUuid {
        parse_uuid(&self.uuid)
    }
    fn timestamp(&self) -> DateTime<Utc> {
        to_dt(self.timestamp)
    }
}
impl Modality for AudioFrame {
    fn get_table_name() -> &'static str {
        "audio"
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
            DEFINE FIELD timestamp     ON `{table}` TYPE datetime;
            DEFINE FIELD audio_bytes  ON `{table}` TYPE bytes;
            DEFINE FIELD codec         ON `{table}` TYPE string;
            DEFINE FIELD sample_rate   ON `{table}` TYPE int;
            DEFINE FIELD channels      ON `{table}` TYPE int;
            DEFINE FIELD duration_secs ON `{table}` TYPE float;
        "#
    }
}

// KeystrokeFrame
impl DataType for KeystrokeFrame {
    fn uuid(&self) -> CoreUuid {
        parse_uuid(&self.uuid)
    }
    fn timestamp(&self) -> DateTime<Utc> {
        to_dt(self.timestamp)
    }
}
impl Modality for KeystrokeFrame {
    fn get_table_name() -> &'static str {
        "keystrokes"
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
            DEFINE FIELD timestamp    ON `{table}` TYPE datetime;
            DEFINE FIELD text         ON `{table}` TYPE string;
            DEFINE FIELD application  ON `{table}` TYPE string;
            DEFINE FIELD window_title ON `{table}` TYPE string;
        "#
    }
}

// ClipboardFrame
impl DataType for ClipboardFrame {
    fn uuid(&self) -> CoreUuid {
        parse_uuid(&self.uuid)
    }
    fn timestamp(&self) -> DateTime<Utc> {
        to_dt(self.timestamp)
    }
}
impl Modality for ClipboardFrame {
    fn get_table_name() -> &'static str {
        "clipboard"
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD text        ON `{table}` TYPE string;
            DEFINE FIELD binary_data ON `{table}` TYPE bytes;
            DEFINE FIELD mime_type   ON `{table}` TYPE string;
        "#
    }
}

// ShellHistoryFrame
impl DataType for ShellHistoryFrame {
    fn uuid(&self) -> CoreUuid {
        parse_uuid(&self.uuid)
    }
    fn timestamp(&self) -> DateTime<Utc> {
        to_dt(self.timestamp)
    }
}
impl Modality for ShellHistoryFrame {
    fn get_table_name() -> &'static str {
        "shell_history"
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
            DEFINE FIELD timestamp   ON `{table}` TYPE datetime;
            DEFINE FIELD command     ON `{table}` TYPE string;
            DEFINE FIELD working_dir ON `{table}` TYPE string;
            DEFINE FIELD exit_code   ON `{table}` TYPE int;
        "#
    }
}

// WindowActivityFrame
impl DataType for WindowActivityFrame {
    fn uuid(&self) -> CoreUuid {
        parse_uuid(&self.uuid)
    }
    fn timestamp(&self) -> DateTime<Utc> {
        to_dt(self.timestamp)
    }
}
impl Modality for WindowActivityFrame {
    fn get_table_name() -> &'static str {
        "window_activity"
    }
    fn get_surrealdb_schema() -> &'static str {
        r#"
            DEFINE FIELD timestamp     ON `{table}` TYPE datetime;
            DEFINE FIELD application   ON `{table}` TYPE string;
            DEFINE FIELD window_title  ON `{table}` TYPE string;
            DEFINE FIELD focused       ON `{table}` TYPE bool;
            DEFINE FIELD duration_secs ON `{table}` TYPE float;
        "#
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::print_stdout)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_timestamp() {
        let ts = Timerange::default();
        let json = serde_json::to_string(&ts).unwrap();
        println!("{}", json);
    }
}
