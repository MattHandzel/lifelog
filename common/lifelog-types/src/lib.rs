use lifelog_core::*;
use lifelog_macros::lifelog_type;
use lifelog_proto::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tokio;

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

#[lifelog_type(None)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorState {
    pub name: String,
    pub timestamp: DateTime<Utc>,
}

#[lifelog_type(None)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceState {}

#[lifelog_type(None)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerState {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub cpu_usage: f32,    // TODO: REFACTOR TO USE USAGE TYPE
    pub memory_usage: f32, // TODO: REFACTOR TO USE USAGE TYPE
    pub threads: f32,      // TODO: REFACTOR TO USE USAGE TYPE

    pub pending_commands: Vec<ServerCommand>,
}

type Query = String;

// TODO: Automatically generate the RPCs for this code so that every action is it's own RPC,
// automatically generate the code for every RPC as they are the exact same code!

#[lifelog_type(None)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerCommand {
    // These are commands to the servers, each of them can result in [0-n] actions. If it is
    // something that can be immediately resolved (such as registering a collector) then it will
    // result in no actions done,
    RegisterCollector,
    GetConfig,
    SetConfig,
    GetData,
    Query,
    ReportState,
    GetState,
}

#[derive(Debug, Clone)]
pub struct ActorConfig;

// TODO: Add all actions to a swervice so any program can tell the server to do anything

#[derive(Debug, Clone)]
pub enum ServerAction {
    Sleep(tokio::time::Duration), // Sleep for a certain amount of time)
    Query(lifelog_proto::QueryRequest),
    GetData(lifelog_proto::GetDataRequest), // TODO: Wouldn't it be cool if the system could specify exactly what data
    // it wanted from the collector so when it has a query it doesn't need to process everything?
    SyncData(Query),
    HealthCheck,
    ReceiveData(Vec<lifelog_proto::Uuid>),
    CompressData(Vec<lifelog_proto::Uuid>),
    TransformData(Vec<lifelog_proto::Uuid>),
    RegisterActor(ActorConfig),
}

impl Default for ServerState {
    fn default() -> Self {
        ServerState {
            name: "LifelogServer".to_string(),
            timestamp: Utc::now(),
            cpu_usage: 0.,    // TODO: REFACTOR TO USE USAGE TYPE
            memory_usage: 0., // TODO: REFACTOR TO USE USAGE TYPE
            threads: 0.,      // TODO: REFACTOR TO USE USAGE TYPE
            pending_commands: vec![],
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/data_modalities.rs"));

#[lifelog_type(None)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LifelogDataKey {
    uuid: String,
}

#[derive(Clone, Debug)]
pub struct RegisteredCollector {
    id: CollectorId,
    address: String,
}

#[derive(Clone, Debug)]
pub struct RegisteredInterface {
    id: InterfaceId,
    address: String,
}

pub type CollectorId = String;
pub type InterfaceId = String;
pub type ServerId = String;

// TODO: We need to model other applications/api's state so they can be used by the server to make
// decisions
#[derive(Clone, Debug)]
pub struct SystemState {
    pub timestamp: DateTime<Utc>,
    pub collector_states: BTreeMap<CollectorId, CollectorState>,
    pub interface_states: BTreeMap<InterfaceId, InterfaceState>,
    pub server_state: ServerState, // There is only 1 server in this model, but maybe we want
                                   // to have more servers in the future
}

impl Default for SystemState {
    fn default() -> Self {
        SystemState {
            timestamp: Utc::now(),
            collector_states: BTreeMap::new(),
            interface_states: BTreeMap::new(),
            server_state: ServerState::default(),
        }
    }
}

#[lifelog_type(None)]
#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DataSource {
    mac: String,            // MAC address of the data source
    modality: DataModality, // the type of data modality
}
