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
pub use lifelog_types::*;
pub use system_state::*;

use lifelog_macros::*;

use dashmap::DashMap;

#[lifelog_type(None)]
#[derive(Debug, Clone)]
pub struct CollectorState {
    name: String,
    timestamp: DateTime<Utc>,
}

#[lifelog_type(None)]
#[derive(Debug, Clone)]
pub struct InterfaceState {}

#[lifelog_type(None)]
#[derive(Clone, Debug)]
pub struct ServerState {
    name: String,
    timestamp: DateTime<Utc>,
}

type CollectorId = String;
type InterfaceId = String;
type ServerId = String;

// TODO: We need to model other applications/api's state so they can be used by the server to make
// decisions
pub struct SystemState {
    pub timestamp: DateTime<Utc>,
    pub collector_states: DashMap<CollectorId, CollectorState>,
    pub interface_states: DashMap<InterfaceId, InterfaceState>,
    pub server_states: DashMap<ServerId, ServerState>, // There is only 1 server in this model, but maybe we want
                                                       // to have more servers in the future
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
