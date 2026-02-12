#![allow(clippy::needless_lifetimes, clippy::large_enum_variant)]

pub mod lifelog {
    include!(concat!(env!("OUT_DIR"), "/lifelog.rs"));
    include!(concat!(env!("OUT_DIR"), "/lifelog.serde.rs"));
}

pub use crate::lifelog::*;

pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("lifelog_descriptor");

#[cfg(feature = "full")]
mod helpers {
    use super::lifelog::*;
    use image::ImageReader;
    use lifelog_core::{
        DataOrigin, DataType, DateTime, LifelogError, LifelogFrameKey, LifelogImage, Modality,
        NaiveDateTime, Utc, Uuid as CoreUuid, Validate,
    };
    use rand::distr::{Distribution, StandardUniform};
    use rand::Rng;
    use std::io::Cursor;

    // --- Helper Functions ---

    pub fn parse_uuid(s: &str) -> CoreUuid {
        s.parse().unwrap_or_else(|_| CoreUuid::nil())
    }

    pub fn to_dt(ts: Option<::pbjson_types::Timestamp>) -> DateTime<Utc> {
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
                        field: "origin".to_string(),
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

    #[cfg(feature = "surrealdb")]
    pub trait ToRecord {
        type Record: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + std::fmt::Debug;
        fn to_record(&self) -> Self::Record;
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct ScreenRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub width: u32,
        pub height: u32,
        pub blob_hash: String,
        pub blob_size: u64,
        pub mime_type: String,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct BrowserRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub url: String,
        pub title: String,
        pub visit_count: u32,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct OcrRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub text: String,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct ProcessRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub processes: Vec<ProcessInfo>,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct CameraRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub width: u32,
        pub height: u32,
        pub blob_hash: String,
        pub blob_size: u64,
        pub mime_type: String,
        pub device: String,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct AudioRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub blob_hash: String,
        pub blob_size: u64,
        pub codec: String,
        pub sample_rate: u32,
        pub channels: u32,
        pub duration_secs: f32,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For audio chunks, this is `t_canonical + duration`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct KeystrokeRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub text: String,
        pub application: String,
        pub window_title: String,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct WeatherRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub temperature: f64,
        pub humidity: f64,
        pub pressure: f64,
        pub conditions: String,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct HyprlandRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub monitors: Vec<HyprMonitor>,
        pub workspaces: Vec<HyprWorkspace>,
        pub active_workspace: Option<HyprWorkspace>,
        pub clients: Vec<HyprClient>,
        pub devices: Vec<HyprDevice>,
        pub cursor: Option<HyprCursor>,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct ClipboardRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub text: String,
        pub binary_data: Vec<u8>,
        /// Optional large clipboard payload stored in CAS; if empty, use `binary_data`.
        pub blob_hash: String,
        pub blob_size: u64,
        pub mime_type: String,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct ShellHistoryRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub command: String,
        pub working_dir: String,
        pub exit_code: i32,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct MouseRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub x: f64,
        pub y: f64,
        pub button: i32,
        pub pressed: bool,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For point records, this equals `t_canonical`.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct WindowActivityRecord {
        pub uuid: String,
        pub timestamp: surrealdb::sql::Datetime,
        pub application: String,
        pub window_title: String,
        pub focused: bool,
        pub duration_secs: f32,
        #[serde(default)]
        pub t_ingest: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub t_canonical: Option<surrealdb::sql::Datetime>,
        /// Canonical end time for interval semantics. For interval records, this is the end time.
        #[serde(default)]
        pub t_end: Option<surrealdb::sql::Datetime>,
        #[serde(default)]
        pub time_quality: Option<String>,
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for ScreenFrame {
        type Record = ScreenRecord;
        fn to_record(&self) -> Self::Record {
            ScreenRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                width: self.width,
                height: self.height,
                blob_hash: String::new(),
                blob_size: self.image_bytes.len() as u64,
                mime_type: self.mime_type.clone(),
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for BrowserFrame {
        type Record = BrowserRecord;
        fn to_record(&self) -> Self::Record {
            BrowserRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                url: self.url.clone(),
                title: self.title.clone(),
                visit_count: self.visit_count,
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for OcrFrame {
        type Record = OcrRecord;
        fn to_record(&self) -> Self::Record {
            OcrRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                text: self.text.clone(),
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for ProcessFrame {
        type Record = ProcessRecord;
        fn to_record(&self) -> Self::Record {
            ProcessRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                processes: self.processes.clone(),
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for CameraFrame {
        type Record = CameraRecord;
        fn to_record(&self) -> Self::Record {
            CameraRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                width: self.width,
                height: self.height,
                blob_hash: String::new(),
                blob_size: self.image_bytes.len() as u64,
                mime_type: self.mime_type.clone(),
                device: self.device.clone(),
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for AudioFrame {
        type Record = AudioRecord;
        fn to_record(&self) -> Self::Record {
            AudioRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                blob_hash: String::new(),
                blob_size: self.audio_bytes.len() as u64,
                codec: self.codec.clone(),
                sample_rate: self.sample_rate,
                channels: self.channels,
                duration_secs: self.duration_secs,
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for KeystrokeFrame {
        type Record = KeystrokeRecord;
        fn to_record(&self) -> Self::Record {
            KeystrokeRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                text: self.text.clone(),
                application: self.application.clone(),
                window_title: self.window_title.clone(),
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for WeatherFrame {
        type Record = WeatherRecord;
        fn to_record(&self) -> Self::Record {
            WeatherRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                temperature: self.temperature,
                humidity: self.humidity,
                pressure: self.pressure,
                conditions: self.conditions.clone(),
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for HyprlandFrame {
        type Record = HyprlandRecord;
        fn to_record(&self) -> Self::Record {
            HyprlandRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                monitors: self.monitors.clone(),
                workspaces: self.workspaces.clone(),
                active_workspace: self.active_workspace.clone(),
                clients: self.clients.clone(),
                devices: self.devices.clone(),
                cursor: self.cursor,
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for ClipboardFrame {
        type Record = ClipboardRecord;
        fn to_record(&self) -> Self::Record {
            ClipboardRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                text: self.text.clone(),
                binary_data: self.binary_data.clone(),
                blob_hash: String::new(),
                blob_size: self.binary_data.len() as u64,
                mime_type: self.mime_type.clone(),
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for ShellHistoryFrame {
        type Record = ShellHistoryRecord;
        fn to_record(&self) -> Self::Record {
            ShellHistoryRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                command: self.command.clone(),
                working_dir: self.working_dir.clone(),
                exit_code: self.exit_code,
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for MouseFrame {
        type Record = MouseRecord;
        fn to_record(&self) -> Self::Record {
            MouseRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                x: self.x,
                y: self.y,
                button: self.button,
                pressed: self.pressed,
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    #[cfg(feature = "surrealdb")]
    impl ToRecord for WindowActivityFrame {
        type Record = WindowActivityRecord;
        fn to_record(&self) -> Self::Record {
            WindowActivityRecord {
                uuid: self.uuid.clone(),
                timestamp: to_dt(self.timestamp).into(),
                application: self.application.clone(),
                window_title: self.window_title.clone(),
                focused: self.focused,
                duration_secs: self.duration_secs,
                t_ingest: None,
                t_canonical: None,
                t_end: None,
                time_quality: None,
            }
        }
    }

    impl Validate for ServerConfig {
        fn validate(&self) -> Result<(), LifelogError> {
            if self.host.is_empty() {
                return Err(LifelogError::Validation {
                    field: "host".to_string(),
                    reason: "must not be empty".to_string(),
                });
            }
            if self.port == 0 || self.port > 65535 {
                return Err(LifelogError::Validation {
                    field: "port".to_string(),
                    reason: format!("must be between 1 and 65535, got {}", self.port),
                });
            }
            if self.database_endpoint.is_empty() {
                return Err(LifelogError::Validation {
                    field: "database_endpoint".to_string(),
                    reason: "must not be empty".to_string(),
                });
            }
            if self.database_name.is_empty() {
                return Err(LifelogError::Validation {
                    field: "database_name".to_string(),
                    reason: "must not be empty".to_string(),
                });
            }
            if self.server_name.is_empty() {
                return Err(LifelogError::Validation {
                    field: "server_name".to_string(),
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
                    field: "id".to_string(),
                    reason: "collector ID must not be empty".to_string(),
                });
            }
            if self.host.is_empty() {
                return Err(LifelogError::Validation {
                    field: "host".to_string(),
                    reason: "must not be empty".to_string(),
                });
            }
            if self.port == 0 || self.port > 65535 {
                return Err(LifelogError::Validation {
                    field: "port".to_string(),
                    reason: format!("must be between 1 and 65535, got {}", self.port),
                });
            }

            if let Some(screen) = &self.screen {
                if screen.enabled && screen.interval <= 0.0 {
                    return Err(LifelogError::Validation {
                        field: "screen.interval".to_string(),
                        reason: "must be positive".to_string(),
                    });
                }
            }

            if let Some(microphone) = &self.microphone {
                if microphone.enabled && microphone.sample_rate == 0 {
                    return Err(LifelogError::Validation {
                        field: "microphone.sample_rate".to_string(),
                        reason: "must be positive".to_string(),
                    });
                }
            }

            Ok(())
        }
    }

    impl Validate for SystemConfig {
        fn validate(&self) -> Result<(), LifelogError> {
            if let Some(server) = &self.server {
                server.validate()?;
            }
            for (id, config) in &self.collectors {
                config.validate().map_err(|e| match e {
                    LifelogError::Validation { field, reason } => LifelogError::Validation {
                        field: format!("collectors[{}].{}", id, field),
                        reason,
                    },
                    _ => e,
                })?;
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
            let reader = match ImageReader::new(Cursor::new(frame.image_bytes))
                .with_guessed_format()
            {
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
            let ts_pb = Some(::pbjson_types::Timestamp {
                seconds: timestamp.timestamp(),
                nanos: timestamp.timestamp_subsec_nanos() as i32,
            });
            ScreenFrame {
                uuid,
                timestamp: ts_pb,
                width,
                height,
                image_bytes: image,
                mime_type: "image/png".to_string(),
                t_device: ts_pb,
                t_canonical: ts_pb,
                t_end: ts_pb,
                ..Default::default()
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
    }

    // MouseFrame
    impl DataType for MouseFrame {
        fn uuid(&self) -> CoreUuid {
            parse_uuid(&self.uuid)
        }
        fn timestamp(&self) -> DateTime<Utc> {
            to_dt(self.timestamp)
        }
    }
    impl Modality for MouseFrame {
        fn get_table_name() -> &'static str {
            "mouse"
        }
    }

    // ProcessFrame
    impl DataType for ProcessFrame {
        fn uuid(&self) -> CoreUuid {
            parse_uuid(&self.uuid)
        }
        fn timestamp(&self) -> DateTime<Utc> {
            to_dt(self.timestamp)
        }
    }
    impl Modality for ProcessFrame {
        fn get_table_name() -> &'static str {
            "processes"
        }
    }

    // CameraFrame
    impl DataType for CameraFrame {
        fn uuid(&self) -> CoreUuid {
            parse_uuid(&self.uuid)
        }
        fn timestamp(&self) -> DateTime<Utc> {
            to_dt(self.timestamp)
        }
    }
    impl Modality for CameraFrame {
        fn get_table_name() -> &'static str {
            "camera"
        }
    }

    // WeatherFrame
    impl DataType for WeatherFrame {
        fn uuid(&self) -> CoreUuid {
            parse_uuid(&self.uuid)
        }
        fn timestamp(&self) -> DateTime<Utc> {
            to_dt(self.timestamp)
        }
    }
    impl Modality for WeatherFrame {
        fn get_table_name() -> &'static str {
            "weather"
        }
    }

    // HyprlandFrame
    impl DataType for HyprlandFrame {
        fn uuid(&self) -> CoreUuid {
            parse_uuid(&self.uuid)
        }
        fn timestamp(&self) -> DateTime<Utc> {
            to_dt(self.timestamp)
        }
    }
    impl Modality for HyprlandFrame {
        fn get_table_name() -> &'static str {
            "hyprland"
        }
    }
}

#[cfg(feature = "full")]
pub use helpers::*;

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::print_stdout)]
mod tests {
    use super::*;
    use lifelog_core::Validate;

    #[test]
    fn test_serialize_timestamp() {
        let ts = Timerange::default();
        let json = serde_json::to_string(&ts).unwrap();
        println!("{}", json);
    }

    #[cfg(feature = "surrealdb")]
    #[test]
    fn test_screen_to_record() {
        let ts_pb = Some(::pbjson_types::Timestamp {
            seconds: 12345,
            nanos: 0,
        });
        let frame = ScreenFrame {
            uuid: lifelog_core::Uuid::new_v4().to_string(),
            timestamp: ts_pb,
            width: 1920,
            height: 1080,
            image_bytes: vec![1, 2, 3],
            mime_type: "image/png".to_string(),
            t_device: ts_pb,
            t_canonical: ts_pb,
            t_end: ts_pb,
            ..Default::default()
        };
        let record = frame.to_record();
        assert_eq!(record.uuid, frame.uuid);
        assert_eq!(record.width, 1920);
        assert_eq!(record.blob_hash, ""); // Placeholder
        assert_eq!(record.blob_size, 3); // vec![1, 2, 3].len()
    }

    #[cfg(feature = "surrealdb")]
    #[test]
    fn test_browser_to_record() {
        let ts_pb = Some(::pbjson_types::Timestamp {
            seconds: 12345,
            nanos: 0,
        });
        let frame = BrowserFrame {
            uuid: lifelog_core::Uuid::new_v4().to_string(),
            timestamp: ts_pb,
            url: "http://test".to_string(),
            title: "title".to_string(),
            visit_count: 5,
            t_device: ts_pb,
            t_canonical: ts_pb,
            t_end: ts_pb,
            ..Default::default()
        };
        let record = frame.to_record();
        assert_eq!(record.url, "http://test");
        assert_eq!(record.visit_count, 5);
    }

    #[cfg(feature = "surrealdb")]
    #[test]
    fn test_mouse_to_record() {
        let ts_pb = Some(::pbjson_types::Timestamp {
            seconds: 12345,
            nanos: 0,
        });
        let frame = MouseFrame {
            uuid: lifelog_core::Uuid::new_v4().to_string(),
            timestamp: ts_pb,
            x: 12.5,
            y: 99.25,
            button: mouse_frame::MouseButton::Left as i32,
            pressed: true,
            t_device: ts_pb,
            t_canonical: ts_pb,
            t_end: ts_pb,
            ..Default::default()
        };
        let record = frame.to_record();
        assert_eq!(record.x, 12.5);
        assert_eq!(record.y, 99.25);
        assert_eq!(record.button, mouse_frame::MouseButton::Left as i32);
        assert!(record.pressed);
    }

    #[test]
    fn test_collector_config_validation() {
        let config = CollectorConfig {
            id: "".to_string(), // Invalid
            ..Default::default()
        };
        assert!(config.validate().is_err());

        let config = CollectorConfig {
            id: "test".to_string(),
            host: "localhost".to_string(),
            port: 8080,
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
