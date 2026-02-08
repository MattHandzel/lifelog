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

/// Core trait for any data modality that can be stored in SurrealDB.
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

    pub fn get_table_name(&self) -> String {
        match &self.origin {
            DataOriginType::DeviceId(device_id) => {
                format!("{}:{}", device_id.replace(":", ""), self.modality_name)
            }
            DataOriginType::DataOrigin(data_origin) => {
                format!("{}:{}", data_origin.get_table_name(), self.modality_name)
            }
        }
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

pub type Result<T, E = LifelogError> = std::result::Result<T, E>;
