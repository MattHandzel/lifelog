use serde::{Deserialize, Serialize};

pub use anyhow;
pub use chrono;
pub use chrono::{DateTime, Utc};
pub use pretty_assertions;
pub use proptest;
pub use serde;
pub use serde_json;
pub use thiserror;
pub use tracing;
pub use uuid;
pub use uuid::Uuid;

pub mod data_sources;
pub mod database_state;
pub mod system_state;
//pub mod system_state;

pub use data_sources::*;
pub use database_state::*;
pub use system_state::*;

pub use tonic;

use dashmap::DashMap;

pub trait DataType {
    fn uuid(&self) -> Uuid;
    fn timestamp(&self) -> DateTime<Utc>;
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum LifelogMacroMetaDataType {
    Config,
    Data,
    None,
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
