use lifelog_core::*;
use lifelog_macros::lifelog_type;
use lifelog_proto::collector_service_client::CollectorServiceClient;
use prost;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use strum_macros::EnumIter;
use thiserror::Error;
use tokio;
use toml;

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
    pub source_states: Vec<String>,
    pub source_buffer_sizes: Vec<String>,
    pub total_buffer_size: u32, // Add to this!! all the information server needs from collector
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
    pub timestamp_of_last_sync: ::lifelog_core::chrono::DateTime<::lifelog_core::chrono::Utc>, // TDOO: REFACTOR TO OPTION and type of f64

    pub pending_actions: Vec<ServerActionType>,
}

type Query = String;

// TODO: Automatically generate the RPCs for this code so that every action is it's own RPC,
// automatically generate the code for every RPC as they are the exact same code!

//#[lifelog_type(None)]
//#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerCommand {
}
//    // These are commands to the servers, each of them can result in [0-n] actions. If it is
//    // something that can be immediately resolved (such as registering a collector) then it will
//    // result in no actions done,
//    RegisterCollector,
//    GetConfig,
//    SetConfig,
//    GetData,
//    Querying, //TODO: Bring this back
//    Transforming,
//    ReportState,
//    GetState,
//}

// TODO: Refactor my system so this is not needed at all! I think it would be cool if the interface
// could not only see that the server is syncing with collectors but also see which one and what
// command it is doing
#[lifelog_type(None)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerActionType {
    Querying,
    GetData, 
    SyncData,
    TransformData,
    CompressData,
    RegisterActor
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
    TransformData(Vec<LifelogFrameKey>),
    SyncData(Query),
    HealthCheck,
    ReceiveData(Vec<lifelog_proto::Uuid>),
    CompressData(Vec<lifelog_proto::Uuid>),
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
            pending_actions: vec![],
            timestamp_of_last_sync: chrono::DateTime::from_timestamp(0, 0)
                .expect("This will never fail"), // TODO: REFACTOR
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/data_modalities.rs"));

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

use std::collections::HashMap;

// TODO: We need to model other applications/api's state so they can be used by the server to make
// decisions
#[lifelog_type(None)]
#[derive(Clone, Debug)]
pub struct SystemState {
    pub collector_states: HashMap<String, CollectorState>,
    pub interface_states: HashMap<String, InterfaceState>,
    pub server_state: ServerState, // There is only 1 server in this model, but maybe we want
                                   // to have more servers in the future
}

impl Default for SystemState {
    fn default() -> Self {
        SystemState {
            collector_states: HashMap::new(),
            interface_states: HashMap::new(),
            server_state: ServerState::default(),
        }
    }
}

#[derive(Debug, Clone, Hash, Deserialize, Serialize)]
pub struct DataSource {
    mac: String,            // MAC address of the data source
    modality: DataModality, // the type of data modality
}

#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Unknown error occurred")]
    Unknown,
}

enum TextTransformationTypes {
    TextEmbedding,
    EntityExtraction,
    KeywordExtraction,
}

enum ImageTransformationTypes {
    OCR,
    ImageEmbedding,
    SensitiveContentDetection,
}

enum TransformType {
    TextEmbedding,
    EntityExtraction,
    OCR,
    ImageEmbedding,
    SensitiveContentDetection,
}

struct TransformConfig {}

struct TransformExampleStruct {
    input: DataSource,
    output: DataSource,
    config: TransformConfig,
}

// TODO: Should this transform's input types and output types by LifelogData types?
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

// TODO: Make this a macro, make folder name automatically be part of struct. Make it so that
// EVERYTHING here is automatically generated (schema as well)
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
    DeviceId(DeviceId), // MAC of device
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
        if parts.len() < 2 {
            panic!("{}", format!("Invalid data origin string: {source}"));
        }
        let modality = DataModality::tryfrom_str(parts[parts.len() - 1]);
        let modality = match modality {
            Ok(modality) => modality,
            Err(e) => {
                return Err(e);
            }
        };
        if parts.len() == 2 {
            return Ok(DataOrigin {
                origin: DataOriginType::DeviceId(parts[0].to_string()),
                modality: modality,
            });
        }
        let potential_origin = DataOrigin::tryfrom_string(parts[0..parts.len() - 1].join(":"));
        match potential_origin {
            Err(e) => {
                return Err(e);
            }
            Ok(origin) => {
                return Ok(DataOrigin {
                    origin: DataOriginType::DataOrigin(Box::new(origin)),
                    modality: modality,
                });
            }
        }
    }

    // TODO: Refactor this to be to_string() function
    pub fn get_table_name(&self) -> String {
        match &self.origin {
            DataOriginType::DeviceId(device_id) => {
                format!(
                    "{}:{}",
                    device_id.replace(":", ""), // Remove ':''s
                    self.modality.to_string()
                )
            }
            DataOriginType::DataOrigin(data_origin) => format!(
                "{}:{}",
                data_origin.get_table_name(),
                self.modality.to_string()
            ),
        }
    }
}

impl fmt::Display for DataOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.origin {
            DataOriginType::DeviceId(device_id) => {
                write!(f, "{}:{}", device_id.replace(":", ""), self.modality)
            }
            DataOriginType::DataOrigin(data_origin) => write!(
                f,
                "{}:{}",
                data_origin.get_table_name(),
                self.modality.to_string()
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

// TODO: Implement from surrealdb error
#[derive(Debug, Error)]
pub enum LifelogError {
    // ─────────────────────────── Infrastructure & IO ──────────────────────────
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    //#[error("database error: {0}")]
    //Database(#[from] surrealdb::Error),
    /// DDL race condition (e.g., `DEFINE TABLE` executed twice concurrently).
    #[error("database: table '{table}' already exists")]
    TableAlreadyExists { table: String },

    #[error("configuration parse error: {0}")]
    Config(#[from] toml::de::Error),

    #[error("database: {0}")]
    Database(String),

    // ────────────────────────────── (De)serialisation ─────────────────────────
    #[error("JSON (de)serialisation error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("protobuf decode error: {0}")]
    ProstDecode(#[from] prost::DecodeError),

    // ───────────────────────── gRPC   ────────────────────────
    #[error("gRPC transport error: {0}")]
    GrpcTransport(#[from] tonic::transport::Error),

    #[error("gRPC status: {0}")]
    GrpcStatus(#[from] tonic::Status),

    // ───────────────────────────── Concurrency layer ──────────────────────────
    #[error("background task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("stream closed unexpectedly")]
    StreamClosed,

    // ───────────────────────────── Domain-specific ────────────────────────────
    #[error("unknown collector '{0}'")]
    UnknownCollector(String),

    #[error("transform '{name}' failed: {source}")]
    Transform {
        name: &'static str,
        #[source]
        source: anyhow::Error,
    },

    #[error("validation failed for field '{field}': {reason}")]
    Validation { field: &'static str, reason: String },

    #[error("tried to parse invalid data modality: '{0}'")]
    InvalidDataModality(String),

    // ────────────────────────────── Catch-all guard ───────────────────────────
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

//// Optional—handy blanket conversion so call-sites can still return `anyhow::Result`.
//impl From<LifelogError> for anyhow::Error {
//    fn from(e: LifelogError) -> Self {
//        anyhow::Error::new(e)
//    }
//}
