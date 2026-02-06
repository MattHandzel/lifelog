pub mod error;
pub mod validate;

use lifelog_core::*;
use lifelog_proto::collector_service_client::CollectorServiceClient;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt;
use tokio;

pub use error::{LifelogError, TransformError};
pub use validate::Validate;

// Re-export state and action types from lifelog_proto
pub use lifelog_proto::{CollectorState, ServerActionType, ServerState, SystemState};

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
pub enum ServerAction {
    Sleep(tokio::time::Duration),
    Query(lifelog_proto::QueryRequest),
    GetData(lifelog_proto::GetDataRequest),
    TransformData(Vec<LifelogFrameKey>),
    SyncData(Query),
    HealthCheck,
    ReceiveData(Vec<lifelog_proto::Uuid>),
    CompressData(Vec<lifelog_proto::Uuid>),
    RegisterActor(ActorConfig),
}

// DataModality is now sourced from lifelog-proto
pub use lifelog_proto::DataModality;

#[derive(Clone, Debug)]
pub struct RegisteredCollector {
    pub id: CollectorId,
    pub address: String,
    pub mac: String,
    pub grpc_client: CollectorServiceClient<tonic::transport::Channel>,
}

#[derive(Clone, Debug)]
pub struct RegisteredInterface {
    pub id: InterfaceId,
    pub address: String,
}

pub type CollectorId = String;
pub type InterfaceId = String;
pub type ServerId = String;

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DataSource {
    mac: String,
    modality: DataModality,
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

pub trait Modality: Sized + Send + Sync + 'static + DeserializeOwned + DataType {
    fn into_payload(self) -> lifelog_proto::lifelog_data::Payload;
    fn get_table_name() -> &'static str;
    fn get_surrealdb_schema() -> &'static str;
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
    pub modality: DataModality,
}

impl DataOrigin {
    pub fn new(source: DataOriginType, modality: DataModality) -> Self {
        DataOrigin {
            origin: source,
            modality,
        }
    }
    pub fn tryfrom_string(source: String) -> Result<Self, LifelogError> {
        let parts = source.split(':').collect::<Vec<_>>();
        match parts[..] {
            [] => Err(LifelogError::InvalidDataModality(source)),
            [_x] => Err(LifelogError::InvalidDataModality(source)),
            [device_id, modality] => Ok(DataOrigin {
                origin: DataOriginType::DeviceId(device_id.to_string()),
                modality: DataModality::from_str_name(modality)
                    .ok_or_else(|| LifelogError::InvalidDataModality(modality.to_string()))?,
            }),
            [.., modality] => {
                let potential_origin =
                    DataOrigin::tryfrom_string(parts[0..parts.len() - 1].join(":"));
                match potential_origin {
                    Err(e) => Err(e),
                    Ok(origin) => Ok(DataOrigin {
                        origin: DataOriginType::DataOrigin(Box::new(origin)),
                        modality: DataModality::from_str_name(modality).ok_or_else(|| {
                            LifelogError::InvalidDataModality(modality.to_string())
                        })?,
                    }),
                }
            }
        }
    }

    pub fn get_table_name(&self) -> String {
        match &self.origin {
            DataOriginType::DeviceId(device_id) => {
                format!(
                    "{}:{}",
                    device_id.replace(":", ""),
                    self.modality.as_str_name()
                )
            }
            DataOriginType::DataOrigin(data_origin) => format!(
                "{}:{}",
                data_origin.get_table_name(),
                self.modality.as_str_name()
            ),
        }
    }
}

impl fmt::Display for DataOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.origin {
            DataOriginType::DeviceId(device_id) => {
                write!(
                    f,
                    "{}:{}",
                    device_id.replace(":", ""),
                    self.modality.as_str_name()
                )
            }
            DataOriginType::DataOrigin(data_origin) => write!(
                f,
                "{}:{}",
                data_origin.get_table_name(),
                self.modality.as_str_name()
            ),
        }
    }
}

pub struct LifelogImage {
    pub uuid: ::lifelog_core::uuid::Uuid,
    pub timestamp: ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>,
    pub image: image::DynamicImage,
}

pub struct LifelogText {
    pub text: String,
    pub uuid: ::lifelog_core::uuid::Uuid,
    pub timestamp: ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>,
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct LifelogFrameKey {
    pub uuid: ::lifelog_core::uuid::Uuid,
    pub origin: DataOrigin,
}

impl From<lifelog_proto::LifelogDataKey> for LifelogFrameKey {
    fn from(key: lifelog_proto::LifelogDataKey) -> Self {
        LifelogFrameKey {
            uuid: key.uuid.parse().expect("unable to parse uuid!"),
            origin: DataOrigin::tryfrom_string(key.origin).unwrap(),
        }
    }
}

impl From<LifelogFrameKey> for lifelog_proto::LifelogDataKey {
    fn from(key: LifelogFrameKey) -> Self {
        lifelog_proto::LifelogDataKey {
            uuid: key.uuid.to_string(),
            origin: key.origin.get_table_name(),
        }
    }
}

impl LifelogFrameKey {
    pub fn new(uuid: ::lifelog_core::uuid::Uuid, origin: DataOrigin) -> Self {
        LifelogFrameKey { uuid, origin }
    }
}
impl fmt::Display for LifelogFrameKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>:<{}>", self.origin.get_table_name(), self.uuid)
    }
}

pub type Result<T, E = LifelogError> = std::result::Result<T, E>;
