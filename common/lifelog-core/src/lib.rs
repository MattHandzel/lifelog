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

pub use serde;
pub use serde::Deserialize;
pub use serde::Serialize;
pub use tonic;

pub mod correlation;
pub mod replay;
pub mod time_skew;

//pub use serde::de::Deserialize;
//pub use serde::ser::Serialize;
//use surrealdb::sql::serde; // TODO: Refactor this please
//
//use serde::{Deserializer, Serializer};
//
///// Serialize a `Uuid` as a plain string (`"550e8400-e29b-41d4-a716-446655440000"`)
//pub fn serialize_uuids<S>(id: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
//where
//    S: Serializer,
//{
//    // `to_hyphenated()` gives the canonical form with dashes
//    serializer.serialize_str(&id.hyphenated().to_string())
//}
//
//use serde::de::{Error as DeError, Unexpected};
//
//pub fn deserialize_uuids<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
//where
//    D: Deserializer<'de>,
//{
//    let s = String::deserialize(deserializer)?;
//    Uuid::parse_str(&s).map_err(|e| {
//        DeError::invalid_value(
//            Unexpected::Str(&s),
//            &format!("a valid UUID: {}", e).as_str(),
//        )
//    })
//}

// TODO: Refactor this trait so it no longer has the `uuid` field. The uuid is the key so it should
// not be stored with the data. This requies a refactor of much more of this project.
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

// use core::{slice, str};
/*
const fn folder_name(path: &str) -> &str {
    let bytes = path.as_bytes();
    let mut last_slash = 0;
    let mut second_last_slash = 0;

    // Same loop logic to find slashes as before
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' || bytes[i] == b'\\' {
            second_last_slash = last_slash;
            last_slash = i;
        }
        i += 1;
    }

    // Calculate slice bounds using pointer arithmetic
    let start = second_last_slash + 1;
    let len = last_slash - start;

    // SAFETY: Original path is valid UTF-8, and we're slicing between valid slash positions
    unsafe {
        let ptr = bytes.as_ptr().add(start);
        let byte_slice = slice::from_raw_parts(ptr, len);
        str::from_utf8_unchecked(byte_slice)
    }
}
*/

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
