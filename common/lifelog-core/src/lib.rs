pub use anyhow;
pub use chrono;
pub use chrono::{DateTime, NaiveDateTime, Utc};
pub use pretty_assertions;
pub use proptest;
pub use serde_json;
pub use thiserror;
pub use tracing;
pub use uuid;
pub use uuid::Uuid;

pub use serde;
pub use serde::Deserialize;
pub use serde::Serialize;
pub use tonic;

pub mod correlation;
pub mod error;
pub mod replay;
pub mod time_skew;
pub mod validate;

pub use error::{LifelogError, TransformError};
pub use validate::Validate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Unit {
    GB,
    Count,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageType {
    Percentage(f32),
    RealValue(u64, Unit),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceState {}

type Query = String;

#[derive(Debug, Clone)]
pub struct ActorConfig;

#[derive(Debug, Clone)]
pub enum ServerAction<Q, G, U> {
    Sleep(tokio::time::Duration),
    Query(Q),
    GetData(G),
    TransformData(Vec<LifelogFrameKey>),
    SyncData(Query),
    HealthCheck,
    ReceiveData(Vec<U>),
    CompressData(Vec<U>),
    RegisterActor(ActorConfig),
}

pub type CollectorId = String;
pub type InterfaceId = String;
pub type ServerId = String;

#[derive(Clone, Debug)]
pub struct RegisteredCollector<CMD, CFG> {
    pub id: CollectorId,
    pub address: String,
    pub mac: String,
    pub command_tx: tokio::sync::mpsc::Sender<Result<CMD, tonic::Status>>,
    pub latest_config: Option<CFG>,
}

#[derive(Clone, Debug)]
pub struct RegisteredInterface {
    pub id: InterfaceId,
    pub address: String,
}

pub trait DataType {
    fn uuid(&self) -> Uuid;
    fn timestamp(&self) -> DateTime<Utc>;
}

/// Core trait for any data modality that can be stored in the database.
pub trait Modality: Sized + Send + Sync + 'static + serde::de::DeserializeOwned + DataType {
    fn get_table_name() -> &'static str;
    fn get_timestamp(&self) -> DateTime<Utc> {
        self.timestamp()
    }
    fn get_uuid(&self) -> Uuid {
        self.uuid()
    }
}

pub type DeviceId = String;

#[derive(Debug, Clone, Hash, Deserialize, Serialize, PartialEq)]
pub enum DataOriginType {
    DeviceId(DeviceId),
    DataOrigin(Box<DataOrigin>),
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize, PartialEq)]
pub struct DataOrigin {
    pub origin: DataOriginType,
    pub modality_name: String, // Stringified modality name to avoid proto dependency
}

impl DataOrigin {
    pub fn new(source: DataOriginType, modality_name: String) -> Self {
        DataOrigin {
            origin: source,
            modality_name,
        }
    }

    pub fn tryfrom_string(source: String) -> Result<Self, LifelogError> {
        let parts = source.split(':').collect::<Vec<_>>();
        match parts[..] {
            [] => Err(LifelogError::InvalidDataModality(source)),
            [_x] => Err(LifelogError::InvalidDataModality(source)),
            [device_id, modality] => Ok(DataOrigin {
                origin: DataOriginType::DeviceId(device_id.to_string()),
                modality_name: modality.to_string(),
            }),
            [.., modality] => {
                let potential_origin =
                    DataOrigin::tryfrom_string(parts[0..parts.len() - 1].join(":"));
                match potential_origin {
                    Err(e) => Err(e),
                    Ok(origin) => Ok(DataOrigin {
                        origin: DataOriginType::DataOrigin(Box::new(origin)),
                        modality_name: modality.to_string(),
                    }),
                }
            }
        }
    }

    pub fn collector_id(&self) -> Option<&str> {
        let mut current = self;
        loop {
            match &current.origin {
                DataOriginType::DeviceId(id) => return Some(id.as_str()),
                DataOriginType::DataOrigin(parent) => current = parent,
            }
        }
    }

    pub fn get_table_name(&self) -> String {
        let mut segments: Vec<String> = vec![self.modality_name.clone()];
        let mut current = self;
        loop {
            match &current.origin {
                DataOriginType::DeviceId(device_id) => {
                    segments.push(device_id.replace(":", ""));
                    break;
                }
                DataOriginType::DataOrigin(inner) => {
                    segments.push(inner.modality_name.clone());
                    current = inner;
                }
            }
        }
        segments.reverse();
        segments.join(":")
    }
}

impl std::fmt::Display for DataOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_table_name())
    }
}

pub struct LifelogImage {
    pub uuid: Uuid,
    pub timestamp: DateTime<Utc>,
    pub image: image::DynamicImage,
}

pub struct LifelogText {
    pub text: String,
    pub uuid: Uuid,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct LifelogFrameKey {
    pub uuid: Uuid,
    pub origin: DataOrigin,
}

impl LifelogFrameKey {
    pub fn new(uuid: Uuid, origin: DataOrigin) -> Self {
        LifelogFrameKey { uuid, origin }
    }
}

impl std::fmt::Display for LifelogFrameKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}>:<{}>", self.origin.get_table_name(), self.uuid)
    }
}

pub trait Transform {
    type Input;
    type Output;
    type Config;

    fn apply(&self, input: Self::Input) -> Result<Self::Output, TransformError>;
    fn source(&self) -> DataOrigin;
    fn destination(&self) -> DataOrigin;
    fn config(&self) -> Self::Config;
    fn new(source: DataOrigin, config: Self::Config) -> Self;

    fn priority(&self) -> u8;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyLevel {
    LocalOnly,
    Zdr,
    #[default]
    Standard,
}

impl std::fmt::Display for PrivacyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LocalOnly => write!(f, "local_only"),
            Self::Zdr => write!(f, "zdr"),
            Self::Standard => write!(f, "standard"),
        }
    }
}

impl std::str::FromStr for PrivacyLevel {
    type Err = LifelogError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "local_only" => Ok(Self::LocalOnly),
            "zdr" => Ok(Self::Zdr),
            "standard" => Ok(Self::Standard),
            other => Err(LifelogError::Other(anyhow::anyhow!(
                "unknown privacy level: {other}"
            ))),
        }
    }
}

impl PrivacyLevel {
    pub fn from_params(params: &std::collections::HashMap<String, String>) -> Self {
        params
            .get("privacy_level")
            .and_then(|s| s.parse().ok())
            .unwrap_or_default()
    }

    pub fn can_process(&self, tier: PrivacyTier) -> bool {
        match tier {
            PrivacyTier::Sensitive => *self == PrivacyLevel::LocalOnly,
            PrivacyTier::Moderate => *self != PrivacyLevel::Standard,
            PrivacyTier::Low => true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyTier {
    Sensitive,
    Moderate,
    Low,
}

impl PrivacyTier {
    pub fn for_modality(modality: &str) -> Self {
        match modality {
            "Keystroke" | "Keystrokes" | "Audio" | "Clipboard" | "Microphone" => {
                PrivacyTier::Sensitive
            }
            "Screen" | "Browser" | "Ocr" | "WindowActivity" | "Camera" => PrivacyTier::Moderate,
            _ => PrivacyTier::Low,
        }
    }
}

impl std::fmt::Display for PrivacyTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sensitive => write!(f, "sensitive"),
            Self::Moderate => write!(f, "moderate"),
            Self::Low => write!(f, "low"),
        }
    }
}

pub type Result<T, E = LifelogError> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privacy_tier_modality_mapping() {
        assert_eq!(
            PrivacyTier::for_modality("Keystrokes"),
            PrivacyTier::Sensitive
        );
        assert_eq!(PrivacyTier::for_modality("Audio"), PrivacyTier::Sensitive);
        assert_eq!(
            PrivacyTier::for_modality("Clipboard"),
            PrivacyTier::Sensitive
        );
        assert_eq!(
            PrivacyTier::for_modality("Microphone"),
            PrivacyTier::Sensitive
        );
        assert_eq!(PrivacyTier::for_modality("Screen"), PrivacyTier::Moderate);
        assert_eq!(PrivacyTier::for_modality("Browser"), PrivacyTier::Moderate);
        assert_eq!(PrivacyTier::for_modality("Weather"), PrivacyTier::Low);
        assert_eq!(PrivacyTier::for_modality("Processes"), PrivacyTier::Low);
    }

    #[test]
    fn privacy_level_enforcement() {
        assert!(PrivacyLevel::LocalOnly.can_process(PrivacyTier::Sensitive));
        assert!(!PrivacyLevel::Zdr.can_process(PrivacyTier::Sensitive));
        assert!(!PrivacyLevel::Standard.can_process(PrivacyTier::Sensitive));

        assert!(PrivacyLevel::LocalOnly.can_process(PrivacyTier::Moderate));
        assert!(PrivacyLevel::Zdr.can_process(PrivacyTier::Moderate));
        assert!(!PrivacyLevel::Standard.can_process(PrivacyTier::Moderate));

        assert!(PrivacyLevel::LocalOnly.can_process(PrivacyTier::Low));
        assert!(PrivacyLevel::Zdr.can_process(PrivacyTier::Low));
        assert!(PrivacyLevel::Standard.can_process(PrivacyTier::Low));
    }

    #[test]
    fn privacy_level_parse_roundtrip() {
        for level in [
            PrivacyLevel::LocalOnly,
            PrivacyLevel::Zdr,
            PrivacyLevel::Standard,
        ] {
            let s = level.to_string();
            let parsed: PrivacyLevel = s.parse().unwrap();
            assert_eq!(parsed, level);
        }
    }
}
