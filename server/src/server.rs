use crate::policy::*;
use chrono::Timelike;
use chrono::Utc;
use config::ServerConfig;
use config::ServerPolicyConfig;
use lifelog_core::*;
use lifelog_types::*;
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use thiserror::Error;
use tokio::sync::RwLock;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status as TonicStatus};

use std::time;
use strum::IntoEnumIterator;

use config::CollectorConfig;
use lifelog_proto::lifelog_server_service_server::LifelogServerService;
use lifelog_proto::{
    GetDataRequest, GetDataResponse, GetStateRequest, GetSystemConfigRequest,
    GetSystemConfigResponse, GetSystemStateResponse, QueryRequest, QueryResponse,
    RegisterCollectorRequest, RegisterCollectorResponse, ReportStateRequest, ReportStateResponse,
    SetSystemConfigRequest, SetSystemConfigResponse,
};
use lifelog_types::DataModality;

use lifelog_proto::collector_service_client::CollectorServiceClient;

use data_modalities::screen::*;
use sysinfo::System;

use futures_core::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use dashmap::DashSet;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use surrealdb::sql::{Thing, Value};

static CREATED_TABLES: Lazy<DashSet<String>> = Lazy::new(DashSet::new);
const SYNC_INTERVAL: i64 = 5; // TODO: Refactor this into policy

// TODO: Refactor into am acro to automatically generate this
const SCREENFRAME_SCHEMA: &str = r#"
    DEFINE FIELD timestamp  ON {table} TYPE datetime;
    DEFINE FIELD width      ON {table} TYPE int;
    DEFINE FIELD height     ON {table} TYPE int;
    DEFINE FIELD image_bytes ON {table} TYPE bytes;
    DEFINE FIELD mime_type  ON {table} TYPE string;
"#;

// TODO: Refactor the timestamp so it uses datetime instead of string, same with uuid, refactor it
// so it doesn't use image_bytes but instead uses a file path to the image
// Maybe have a custom struct for data retrievals and parse it to the actual representation?
// So when we get a struct ScreenFrame from the client, we might want to store the file onto some
// database, get that file path, create a new struct ScreenFrameSurreal that has the surrealdb
// types?

async fn ensure_table(
    db: &Surreal<Client>,
    table: &str,
    schema_tpl: &str,
) -> surrealdb::Result<()> {
    if CREATED_TABLES.contains(table) {
        return Ok(());
    }
    let ddl = schema_tpl.replace("{table}", table);
    //db.query(format!(
    //    r#"
    //    DEFINE TABLE {table} SCHEMAFULL;
    //    {ddl}
    //    DEFINE INDEX {table}_ts_idx ON {table} FIELDS timestamp;
    //"#
    //))
    // TODO: we want to be able to define the index as well
    db.query(format!(
        r#"
        DEFINE TABLE {table} SCHEMAFULL;
        {ddl}
    "#
    ))
    .await?;
    CREATED_TABLES.insert(table.to_owned());
    Ok(())
}

//type Loader = fn(&Surreal<Client>, &[Uuid]) -> anyhow::Result<Vec<LifelogData>>;

//static MODALITY_REGISTRY: Lazy<DashMap<DataModality, Loader>> =
//    Lazy::new(|| DashMap::from([(DataModality::Screen, load::<ScreenFrame> as Loader)]));

//async fn load<T: Modality>(
//    db: &Surreal<Client>,
//    uuids: &[Uuid],
//) -> anyhow::Result<Vec<LifelogData>> {
//    let rows: Vec<T> = db.select::<Vec<T>>(T::TABLE).await?; // filter by uuids
//    Ok(rows
//        .into_iter()
//        .map(|r| LifelogData {
//            payload: Some(r.into_payload()),
//        })
//        .collect())
//}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Tonic transport error: {0}")]
    TonicError(#[from] tonic::transport::Error),
    #[error("Internal error: {0}")]
    InternalError(String),
}

// Convert server commands to action

//impl ServerCommand {
//    pub fn to_actions(&self) -> Vec<ServerAction> {
//        match self {
//            ServerCommand::RegisterCollector(request) => {
//                vec![]
//            }
//            ServerCommand::GetConfig(_) => {
//                vec![]
//            }
//            ServerCommand::SetConfig(_) => vec![],
//            ServerCommand::Query(request) => vec![ServerAction::Query(request.clone())],
//            _ => {
//                panic!("Command {:?} not implemented yet", self);
//            }
//        }
//    }
//}

#[derive(Clone)]
pub struct ServerHandle {
    pub server: Arc<RwLock<Server>>,
}

impl ServerHandle {
    pub fn new(server: Arc<RwLock<Server>>) -> Self {
        Self { server: server }
    }

    pub async fn get_state(&self) -> SystemState {
        let server = self.server.read().await;
        server.get_state().await // TODO: This could be refacotred to just use the system state
                                 // instead of recomputing the state. There is a discrepency in use.
    }
    pub async fn get_policy(&self) -> Arc<RwLock<ServerPolicy>> {
        let server = self.server.read().await;
        server.get_policy()
    }

    pub async fn r#loop(&self) {
        loop {
            let server = self.server.write().await;
            server.step().await;
            tokio::time::sleep(time::Duration::from_millis(100)).await;
        }
    }

    pub async fn get_db(&self) -> Surreal<Client> {
        let server = self.server.read().await;
        server.db.clone()
    }

    // TODO: Refactor having to define all of these functions... I am not a fan of them...
    pub async fn register_collector(&self, collector: RegisteredCollector) {
        let mut server = self.server.write().await;
        server.register_collectors.write().await.push(collector);
    }

    pub async fn contains_collector(&self, collector_name: String) -> bool {
        let server = self.server.read().await;
        server.contains_collector(collector_name).await
    }
}

// TDOO: ADD A CHANNEL FOR COMMS
#[derive(Debug, Clone)]
pub struct Server {
    db: Surreal<Client>,
    host: String,
    port: u16,
    state: Arc<RwLock<SystemState>>,
    register_collectors: Arc<RwLock<Vec<RegisteredCollector>>>,
    register_interfaces: Arc<RwLock<Vec<RegisteredInterface>>>,
    policy: Arc<RwLock<ServerPolicy>>,
}

static mut SYS: Option<sysinfo::System> = None;

const SERVER_COMMAND_CHANNEL_BUFFER_SIZE: usize = 100;

impl Server {
    pub async fn new(config: &ServerConfig) -> Result<Self, ServerError> {
        let db = Surreal::new::<Ws>(&config.database_endpoint).await.expect("Could not connect to the database, do you have it running? surreal start --user root --pass root --log debug rocksdb://~/lifelog/database --bind \"127.0.0.1:7183\"");
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await?;

        db.use_ns("lifelog")
            .use_db(config.database_name.clone())
            .await?;

        let state = SystemState {
            server_state: ServerState {
                name: config.server_name.clone(),
                ..Default::default()
            },
            ..Default::default()
        };
        let state = Arc::new(RwLock::new(state));

        let policy = Arc::new(RwLock::new(ServerPolicy {
            config: ServerPolicyConfig::default(),
        }));
        unsafe {
            SYS = Some(System::new_all());
        }

        let s = Self {
            db,
            host: config.host.clone(),
            port: config.port as u16,
            state,
            register_collectors: Arc::new(RwLock::new(vec![])),
            register_interfaces: Arc::new(RwLock::new(vec![])),
            policy,
        };

        Ok(s)
    }
}

pub struct GRPCServerLifelogServerService {
    pub server: ServerHandle,
}

#[tonic::async_trait]
impl LifelogServerService for GRPCServerLifelogServerService {
    async fn register_collector(
        &self,
        request: TonicRequest<RegisterCollectorRequest>,
    ) -> Result<TonicResponse<RegisterCollectorResponse>, TonicStatus> {
        let inner = request.into_inner();
        let collector_config: CollectorConfig = inner.config.unwrap().into();
        let collector_ip = format!(
            "http://{}:{}", // TODO: I shouldn't explicitly write http here, it should either be
            // defined in the config or the protocol should be
            collector_config.host.clone(),
            collector_config.port.clone()
        );
        println!(
            "Received a register collector request from: {:?}",
            collector_ip
        );

        let endpoint = tonic::transport::Endpoint::from_shared(collector_ip.clone());
        match endpoint {
            Err(ref e) => {
                println!("Endpoint: {:?}", endpoint);
                Err(TonicStatus::internal(format!(
                    "Failed to create endpoint: {}",
                    e
                )))
            }
            Ok(endpoint) => {
                let endpoint = endpoint.connect_timeout(time::Duration::from_secs(10));

                let channel = endpoint.connect().await.map_err(|e| {
                    TonicStatus::internal(format!("Failed to connect to endpoint: {}", e))
                })?;
                let client = CollectorServiceClient::new(channel);

                let collector = RegisteredCollector {
                    id: collector_config.id.clone(),
                    mac: "FF:FF:FF:FF:FF:FF".to_string().replace(":", ""), // TODO: Implement this on the collector
                    address: collector_ip.to_string(),
                    grpc_client: client.clone(),
                };
                self.server.register_collector(collector.clone()).await;
                println!("Registering collector: {:?}", collector);

                Ok(TonicResponse::new(RegisterCollectorResponse {
                    success: true,
                    session_id: chrono::Utc::now().timestamp_subsec_nanos() as u64
                        + chrono::Utc::now().timestamp() as u64,
                }))
            }
        }
    }

    async fn get_config(
        &self,
        _request: tonic::Request<GetSystemConfigRequest>,
    ) -> Result<TonicResponse<GetSystemConfigResponse>, TonicStatus> {
        println!("Received a get config request!");
        Ok(TonicResponse::new(GetSystemConfigResponse::default()))
    }

    async fn set_config(
        &self,
        _request: tonic::Request<SetSystemConfigRequest>,
    ) -> Result<TonicResponse<SetSystemConfigResponse>, TonicStatus> {
        println!("Received a set config request!");
        Ok(TonicResponse::new(SetSystemConfigResponse::default()))
    }

    async fn get_data(
        &self,
        request: TonicRequest<GetDataRequest>,
    ) -> Result<TonicResponse<GetDataResponse>, TonicStatus> {
        let req = request.into_inner();
        let chunks: Vec<lifelog_proto::lifelog_data::Payload> = Vec::new();
        for modality in DataModality::iter() {
            // Find the uuids in the data base
        }
        // 4) turn it into a gRPC response
        //match result {
        //Ok(resp) => Ok(TonicResponse::new(resp)),
        //    Err(stat) => Err(TonicStatus::internal(stat.to_string())),
        //}
        Ok(TonicResponse::new(GetDataResponse { data: vec![] }))
    }

    // TODO: Refactor ALL functions to include this check to see if the thing doing the requesting
    // has been registered or not. I want to refactor my server to only think about clients.
    async fn report_state(
        &self,
        _request: tonic::Request<ReportStateRequest>,
    ) -> Result<TonicResponse<ReportStateResponse>, TonicStatus> {
        let state = _request.into_inner().state.unwrap();
        println!(
            "Received a get state request! {} {:?}",
            state.name,
            state.timestamp.unwrap()
        );
        // Ensure we got a state reported by a registered collector, if not then we ignore it
        match self.server.contains_collector(state.name.clone()).await {
            true => Ok(TonicResponse::new(ReportStateResponse {
                acknowledged: true,
            })),
            false => Err(TonicStatus::internal(format!(
                "Collector {} is not registered",
                state.name
            ))),
        }
    }

    async fn query(
        &self,
        request: tonic::Request<QueryRequest>,
    ) -> Result<tonic::Response<QueryResponse>, tonic::Status> {
        let QueryRequest { query } = request.into_inner();
        if query.trim().is_empty() {
            return Err(tonic::Status::invalid_argument("query string is empty"));
        }

        let db = self.server.get_db().await;
        println!("Query: {}", query);
        let mut rows: Vec<ScreenFrameSurreal> = db
            .select(query)
            .await
            .map_err(|e| tonic::Status::internal(format!("database error: {e}")))?;
        //println!("Response: {:?}", resp);

        let mut response = QueryResponse { uuids: vec![] };

        for v in rows {
            response.uuids.push(v.timestamp.to_string());
        }

        Ok(tonic::Response::new(response))
    }
    async fn get_state(
        &self,
        _request: tonic::Request<GetStateRequest>,
    ) -> Result<TonicResponse<GetSystemStateResponse>, TonicStatus> {
        println!("Received a get state request!");
        let state = self.server.get_state().await;
        Ok(TonicResponse::new(GetSystemStateResponse {
            state: Some(state.server_state.into()), // TODO: Replace this with the system state
                                                    // instead of the server state (i need some work with the proto files)
        }))
    }
}

#[derive(Debug, Clone)]
pub struct ServerPolicy {
    config: ServerPolicyConfig,
}

impl Policy for ServerPolicy {
    type StateType = SystemState;
    type ActionType = ServerAction;

    fn get_action(&self, state: &Self::StateType) -> Self::ActionType {
        // Logic to decide the action
        // For example, look at the history, what actions we are already doing

        // Look at the last time different maintenance actions were performed

        // See what the current client requests are

        // TODO: Look at the collector states and when a collector is more than x seconds out of
        // sync ask it for more data
        let action = if (state.server_state.timestamp - state.server_state.timestamp_of_last_sync)
            .num_seconds() as f64
            >= (self.config.collector_sync_interval as f64)
        // TODO: Refactor so this happens once
        {
            // TODO: Add the specific data modality here
            ServerAction::SyncData("SELECT * FROM screen".to_string()) // TODO: Refactor so that we
                                                                       // can sync from all collectors
        } else {
            ServerAction::Sleep(tokio::time::Duration::from_millis(100))
        };

        action
    }
}

// TODO: refactor the server to be an actor model instead of RwLock
// https://softwarepatternslexicon.com/patterns-rust/9/14/
impl Server {
    /// This function will be run upon startup, it will handle the server's main loop of doing
    /// actions

    // TODO: Refactor this so do_action (a blocking task) isn't running here, we don't wanna hold
    // onto the lock
    pub async fn step(&self) -> () {
        let state = self.get_state().await;
        let action = self.policy.read().await.get_action(&state); // TODO: REFACTOR this so policy is
                                                                  // normal variable

        // Perform the action
        // TODO: Czy mam problem że akcje byś mogły trwać długo? Jak to rozwiązać

        // TODO: dodaj parallelizm do tego, żeby nie czekać na zakończenie akcji
        //tokio::task::spawn(async move {
        //Add to audit log
        self.add_audit_log(&action).await;
        self.do_action(action, state).await;
    }

    fn get_policy(&self) -> Arc<RwLock<ServerPolicy>> {
        // Get the policy from the config
        return self.policy.clone();
    }

    // NOTE: This function makes an assumption that each collector's name is unique. If the
    // collector has a different name then they are different collectors, same name means same
    // collector. Are there any problems with this?
    async fn contains_collector(&self, collector_name: String) -> bool {
        let collectors = self.register_collectors.read().await;
        println!(
            "Checking if collector {} is registered: {:?}",
            collector_name, collectors
        );
        for collector in collectors.iter() {
            if collector.id == collector_name {
                return true;
            }
        }
        false
    }

    async fn get_state(&self) -> SystemState {
        unsafe {
            // TODO: refactor this to be safe
            let sys = SYS.as_mut().expect("System info is not initialized");
            sys.refresh_all();

            let cpu_usage = (sys.global_cpu_usage() as f32) / 100.0; // [0-1]

            let total_mem = sys.total_memory() as f32; // KiB
            let used_mem = sys.used_memory() as f32;
            let memory_usage = if total_mem > 0.0 {
                used_mem / total_mem
            } else {
                0.0
            };

            let processes = sys.processes().len() as i32;

            //let total_disk: u64 = sys.disks().iter().map(|d| d.total_space()).sum();
            //let free_disk: u64 = sys.disks().iter().map(|d| d.available_space()).sum();
            //let disk_usage = if total_disk > 0 {
            //    (total_disk - free_disk) as f64 / total_disk as f64
            //} else {
            //    0.0
            //};

            // Estimate the state in this function
            let mut state = self.state.write().await;
            state.server_state.cpu_usage = cpu_usage; // TODO: Get the real CPU usage
            state.server_state.timestamp = Utc::now();
            state.server_state.memory_usage = memory_usage; // TODO: Get the real memory usage
                                                            // TODO: Get the real number of threads
                                                            // TODO: There is a race condition here, someone can grab the lock before we can grab it
            state.clone()
        }
    }

    async fn add_audit_log(&self, action: &ServerAction) {
        //println!("Adding audit log for action: {:?}", action);
    }

    // TODO: Maybe i can use rayon for automatic parallelism?
    async fn do_action(&self, action: ServerAction, state: SystemState) {
        // Perform the action
        match action {
            ServerAction::Sleep(duration) => {
                // Sleep for a certain amount of time
                tokio::time::sleep(duration).await;
            }
            ServerAction::SyncData(query) => {
                // TODO: Refactor so we actually use the query
                // Get the target data modalities(s) from the query
                let mut collectors = self.register_collectors.write().await;
                for collector in collectors.iter_mut() {
                    println!("Syncing data with collector: {:?}", collector);
                    // TODO: This code can fail here (notice the unwraps, I should handle it.
                    let mut stream = collector
                        .grpc_client
                        .get_data(GetDataRequest {
                            uuids: vec![query.clone().into()],
                        })
                        .await
                        .unwrap()
                        .into_inner();
                    println!("Defined the stream here...");
                    let mut data = vec![];

                    while let Some(chunk) = stream.next().await {
                        data.push(chunk.unwrap().payload);
                    }
                    println!("Done receiving data");
                    let mac = collector.mac.clone();

                    // TODO: REFACTOR THIS FUNCTION

                    let data_modality_str = "screen";
                    let table = format!("`{}:{}`", mac, data_modality_str);

                    ensure_table(&self.db, &table, SCREENFRAME_SCHEMA)
                        .await
                        .unwrap();
                    for chunk in data {
                        // record id = random UUID
                        let chunk = chunk.unwrap();

                        // TODO: Repeat for every data type, just do the old c.into()
                        let mut chunk: ScreenFrame = match chunk {
                            lifelog_proto::lifelog_data::Payload::Screenframe(c) => c.into(),
                            _ => unimplemented!(),
                        };
                        chunk.timestamp = chunk
                            .timestamp
                            .with_nanosecond(chunk.timestamp.timestamp_subsec_micros() * 1000)
                            .unwrap();
                        println!("Adding {:?} to table {}", chunk.uuid, table);

                        let uuid = chunk.uuid;
                        let chunk: ScreenFrameSurreal = chunk.into();
                        let _: ScreenFrameSurreal = self
                            .db
                            .create((table.as_str(), uuid.to_string()))
                            .content(chunk)
                            .await
                            .unwrap()
                            .expect(
                                format!("Unable to create row in table {}", table.as_str())
                                    .as_str(),
                            );

                        //db.create("audit_log")
                        //  .content(json!({
                        //      "ts": chrono::Utc::now(),
                        //      "actor": msg.source_mac,
                        //      "action": "ingest",
                        //      "detail": { "table": table }
                        //  }))
                        //  .await?;
                    }

                    // TODO: refactor, i dont think we should write lock the state here, diff
                    // function for estimating the state?
                    self.state.write().await.server_state.timestamp_of_last_sync = Utc::now();
                }

                // For now, assume we want to sync all data modalities

                // Ask the collectors to send data
            }
            _ => todo!(),
        }
    }

    //pub async fn get_tables(db: &Surreal<Client>) -> surrealdb::Result<Vec<String>> {
    //    #[derive(serde::Deserialize)]
    //    struct Info {
    //        tables: std::collections::HashMap<String, serde_json::Value>,
    //    }
    //
    //    let info: Info = db.query("INFO FOR DB").await?.take(0)?;
    //    Ok(info.tables.keys().cloned().collect())
    //}
    //
    //pub async fn fetch_latest<T>(
    //    db: &Surreal<Client>,
    //    table: &str,
    //    limit: u32,
    //) -> surrealdb::Result<Vec<T>>
    //where
    //    T: DeserializeOwned,
    //{
    //    db.query(format!(
    //        "SELECT * FROM {table} ORDER BY timestamp DESC LIMIT {limit}"
    //    ))
    //    .await?
    //    .take(0)
    //}

    //pub async fn fetch_by_id<T>(
    //    db: &Surreal<Client>,
    //    table: &str,
    //    id: &str,
    //) -> surrealdb::Result<Option<T>>
    //where
    //    T: DeserializeOwned,
    //{
    //    let rid = Thing::from((table, id));
    //    db.select((table, id)).await
    //}
    //pub async fn count_table(db: &Surreal<Client>, table: &str) -> surrealdb::Result<u64> {
    //    #[derive(serde::Deserialize)]
    //    struct Cnt {
    //        count: u64,
    //    }
    //    let cnt: Cnt = db
    //        .query(format!("SELECT count() AS count FROM {table}"))
    //        .await?
    //        .take(0)?;
    //    Ok(cnt.count)
    //}
    //pub async fn live_select(
    //    db: &Surreal<Client>,
    //    table: &str,
    //) -> surrealdb::Result<impl Stream<Item = surrealdb::Result<serde_json::Value>>> {
    //    db.select_live(format!("LIVE SELECT * FROM {table}")).await
    //}
}

// TODO: Complete this for every data type
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScreenFrameSurreal {
    //pub uuid: String,
    pub timestamp: surrealdb::Datetime,
    pub width: i32,
    pub height: i32,
    pub image_bytes: surrealdb::sql::Bytes,
    pub mime_type: String,
}

// TDOO: Do this for every datatype
impl From<ScreenFrame> for ScreenFrameSurreal {
    fn from(frame: ScreenFrame) -> Self {
        Self {
            //uuid: frame.uuid.into(),
            timestamp: frame.timestamp.into(),
            width: frame.width as i32,
            height: frame.height as i32,
            image_bytes: frame.image_bytes.into(),
            mime_type: frame.mime_type,
        }
    }
}
