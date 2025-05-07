pub use anyhow;
pub use chrono;
pub use chrono::{DateTime, Utc};
pub use pretty_assertions;
pub use proptest;
//pub use serde;
pub use serde_json;
pub use thiserror;
pub use tracing;
pub use uuid;
pub use uuid::Uuid;

pub mod data_sources;
pub mod database_state;
pub mod system_state;
//pub mod system_state;

pub use serde;
pub use serde::Deserialize;
pub use serde::Serialize;
pub use tonic;

//pub use serde::de::Deserialize;
//pub use serde::ser::Serialize;
//use surrealdb::sql::serde; // TODO: Refactor this please
//
use serde::{Deserializer, Serializer};

/// Serialize a `Uuid` as a plain string (`"550e8400-e29b-41d4-a716-446655440000"`)
pub fn serialize_uuids<S>(id: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // `to_hyphenated()` gives the canonical form with dashes
    serializer.serialize_str(&id.hyphenated().to_string())
}

use serde::de::{Error as DeError, Unexpected};

pub fn deserialize_uuids<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Uuid::parse_str(&s).map_err(|e| {
        DeError::invalid_value(
            Unexpected::Str(&s),
            &format!("a valid UUID: {}", e).as_str(),
        )
    })
}

pub trait DataType {
    fn uuid(&self) -> Uuid;
    fn timestamp(&self) -> DateTime<Utc>;

    // TODO:
    // fn schema
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LifelogMacroMetaDataType {
    Config,
    Data,
    None,
}

use ::serde::de::DeserializeOwned;
pub trait Modality: Sized + Send + Sync + 'static + DeserializeOwned {
    const TABLE: &'static str;
    fn into_payload(self) -> lifelog_proto::lifelog_data::Payload;
    fn id(&self) -> String;
}

//use system_state::*;

//use target_lexicon::Triple as ComputerTargetTriple;
//
//enum PhoneType {
//    Android(AndroidOperatingSystem),
//    IPhone(IPhoneOperatingSystem),
//}
//
//enum ComputerType {
//    Desktop(ComputerTargetTriple),
//    Laptop(ComputerTargetTriple),
//}
//
//enum DeviceType {
//    Phone(PhoneType),
//    Computer(ComputerType),
//}
//
//struct Collector {
//    name: String,                                      // name of the collector
//    device: DeviceType,                                // type of device (phone, computer, etc)
//    location: URI, // location of the collector (ip address, bluetooth address, etc)
//    config: CollectorConfig, // configuration of the collector
//    state: CollectorState, // state of the collector (state of the collector and all data sources, loggers)
//    data_sources: DashMap<DataSourceType, DataSource>, // data sources available on the device
//    grpc_client: GrpcClient, // gRPC client to communicate with the server
//    security_context: SecurityContext, // security context to ensure the data being sent is not tampered with
//    command_tx: mpsc::Sender<CollectorCommand>, // commands to send between threads
//    command_rx: mpsc::Receiver<CollectorCommand>, // commands to send between threads
//}
//
