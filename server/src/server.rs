use crate::policy::*;
use chrono::Timelike;
use chrono::Utc;
use config::ServerPolicyConfig;
use config::{ServerConfig, SystemConfig};
use lifelog_core::*;
use lifelog_types::*;
use std::sync::Arc;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use thiserror::Error;
use tokio::sync::RwLock;
use tonic::{Request as TonicRequest, Response as TonicResponse, Status as TonicStatus};

use std::collections::HashMap;
use std::time;
use strum::IntoEnumIterator;

use config::CollectorConfig;
use lifelog_proto::lifelog_server_service_server::LifelogServerService;
use lifelog_proto::{
    GetDataRequest, GetDataResponse, GetStateRequest, GetSystemConfigRequest,
    GetSystemConfigResponse, GetSystemStateResponse, Query, QueryRequest, QueryResponse,
    RegisterCollectorRequest, RegisterCollectorResponse, ReportStateRequest, ReportStateResponse,
    SetSystemConfigRequest, SetSystemConfigResponse, Timerange,
};
use lifelog_types::DataModality;

use lifelog_proto::collector_service_client::CollectorServiceClient;

use data_modalities::*;
use prost_types::Timestamp;
use std::collections::HashSet;
use sysinfo::System;

use futures_core::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use dashmap::DashSet;
use once_cell::sync::Lazy;
use serde::{de::DeserializeOwned, Serialize};
use surrealdb::sql::{Thing, Value};
use surrealdb::Error;

static CREATED_TABLES: Lazy<DashSet<String>> = Lazy::new(DashSet::new);
const SYNC_INTERVAL: i64 = 5; // TODO: Refactor this into policy

// TODO: Refactor the timestamp so it uses datetime instead of string, same with uuid, refactor it
// so it doesn't use image_bytes but instead uses a file path to the image
// Maybe have a custom struct for data retrievals and parse it to the actual representation?
// So when we get a struct ScreenFrame from the client, we might want to store the file onto some
// database, get that file path, create a new struct ScreenFrameSurreal that has the surrealdb
// types?

async fn ensure_table(db: &Surreal<Client>, data_origin: &DataOrigin) -> surrealdb::Result<()> {
    let table = data_origin.get_table_name();
    if CREATED_TABLES.contains(&table) {
        return Ok(());
    }
    // TODO: Auto generate this or find a better way of representing
    let schema_tpl = match data_origin.modality {
        DataModality::Screen => ScreenFrame::get_surrealdb_schema(),
        DataModality::Browser => BrowserFrame::get_surrealdb_schema(),
        DataModality::Ocr => OcrFrame::get_surrealdb_schema(),
        _ => unimplemented!(),
    };
    let ddl = schema_tpl.replace("{table}", &table);
    //db.query(format!(
    //    r#"
    //    DEFINE TABLE {table} SCHEMAFULL;
    //    {ddl}
    //    DEFINE INDEX {table}_ts_idx ON {table} FIELDS timestamp;
    //"#
    //))
    // TODO: we want to be able to define the index as well
    let db_query = format!(
        r#"
        DEFINE TABLE `{table}` SCHEMAFULL;
        {ddl}
    "#
    );
    db.query(db_query.clone()).await?;
    CREATED_TABLES.insert(table.to_owned());
    println!("Ensuring table schema: {}", db_query);
    println!("Ensuring table: {}", table);
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
            {
                //println!("[LOOP]: Requesting server lock");
                let server = self.server.write().await;
                server.step().await;
            }
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
        server.registered_collectors.write().await.push(collector);
    }

    pub async fn contains_collector(&self, collector_name: String) -> bool {
        let server = self.server.read().await;
        server.contains_collector(collector_name).await
    }

    pub async fn report_collector_state(&self, state: CollectorState) -> Result<(), LifelogError> {
        let mut server = self.server.write().await;
        server.report_collector_state(state).await
    }

    pub async fn process_query(&self, query: String) -> Result<Vec<LifelogFrameKey>, LifelogError> {
        println!(
            "[PROCESS_QUERY]: Requesting server lock for query {}",
            query
        );
        self.server.read().await.process_query(query).await
    }

    pub async fn get_system_config(&self) -> Result<SystemConfig, LifelogError> {
        let server = self.server.read().await;
        server.get_system_config().await
    }

    pub async fn get_data(
        &self,
        keys: Vec<LifelogFrameKey>,
    ) -> Result<Vec<LifelogData>, LifelogError> {
        let server = self.server.read().await;
        server.get_data(keys).await
    }
}

// TDOO: ADD A CHANNEL FOR COMMS
#[derive(Debug, Clone)]
pub struct Server {
    db: Surreal<Client>,
    host: String,
    port: u16,
    state: Arc<RwLock<SystemState>>,
    registered_collectors: Arc<RwLock<Vec<RegisteredCollector>>>,
    register_interfaces: Arc<RwLock<Vec<RegisteredInterface>>>,
    policy: Arc<RwLock<ServerPolicy>>,
    origins: Arc<RwLock<Vec<DataOrigin>>>,
    transforms: Arc<RwLock<Vec<LifelogTransform>>>, // TODO: These should be registered transforms
    config: ServerConfig,
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

        let ocr_transform = OcrTransform::new(
            DataOrigin::new(
                DataOriginType::DeviceId("FF:FF:FF:FF:FF:FF".to_string()),
                DataModality::Screen,
            ),
            OcrConfig {
                language: "eng".to_string(),
                engine_path: None,
            },
        );

        let origins_vec = get_origins_from_db(&db)
            .await
            .expect("Failed to get origins from db");
        println!("[INSTANTIATION]: Origins: {:?}", origins_vec);

        let s = Self {
            db,
            host: config.host.clone(),
            port: config.port as u16,
            state,
            registered_collectors: Arc::new(RwLock::new(vec![])),
            register_interfaces: Arc::new(RwLock::new(vec![])),
            policy,
            transforms: Arc::new(RwLock::new(vec![ocr_transform.into()])),
            origins: Arc::new(RwLock::new(origins_vec)),
            config: config.clone(),
        };

        Ok(s)
    }
    async fn get_system_config(&self) -> Result<SystemConfig, LifelogError> {
        // Get the config of all the collectors
        let mut collectors = self.registered_collectors.write().await;
        let mut collector_configs: HashMap<String, CollectorConfig> = HashMap::new();
        for collector in collectors.iter_mut() {
            let config: CollectorConfig = collector
                .grpc_client
                .get_config(lifelog_proto::GetCollectorConfigRequest {})
                .await?
                .into_inner()
                .config
                .unwrap() // TODO: Instead of panicing we should get reutrn null
                .into();
            collector_configs.insert(collector.id.clone(), config);
        }
        let config = SystemConfig {
            server: self.config.clone(),
            collectors: collector_configs,
        };

        Ok(config)
    }
    async fn report_collector_state(&self, state: CollectorState) -> Result<(), LifelogError> {
        let mut system_state = self.state.write().await;
        system_state
            .collector_states
            .insert(state.name.clone(), state);
        Ok(())
    }

    async fn get_data(&self, req: Vec<LifelogFrameKey>) -> Result<Vec<LifelogData>, LifelogError> {
        let mut datas: Vec<LifelogData> = vec![];
        for key in req.iter() {
            let data: LifelogData = get_data_by_key(&self.db, key) // TODO: Refactor this so it's faster,
                // less db queries
                .await
                .expect(format!("Unable to get data by key: {}", key).as_str());

            datas.push(data);
        }
        Ok(datas)
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
                // Add origins to our origins

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
        let system_config =
            self.server.get_system_config().await.map_err(|e| {
                TonicStatus::internal(format!("Failed to get system config: {}", e))
            })?;
        Ok(TonicResponse::new(GetSystemConfigResponse {
            config: Some(system_config.into()),
        }))
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
        let GetDataRequest { keys } = req;
        let data: Vec<lifelog_proto::LifelogData> = self
            .server
            .get_data(
                keys.iter()
                    .map(|k| LifelogFrameKey::from(k.clone()))
                    .collect(),
            )
            .await
            .map_err(|e| TonicStatus::internal(format!("Failed to get data: {}", e)))?
            .iter()
            .map(|v| lifelog_proto::LifelogData::from(v.clone()))
            .collect();

        Ok(TonicResponse::new(GetDataResponse { data: data }))
    }

    // TODO: Refactor ALL functions to include this check to see if the thing doing the requesting
    // has been registered or not. I want to refactor my server to only think about clients.
    async fn report_state(
        &self,
        _request: tonic::Request<ReportStateRequest>,
    ) -> Result<TonicResponse<ReportStateResponse>, TonicStatus> {
        let state = _request.into_inner().state.unwrap();
        println!(
            "Received a report state request! {} {:?}",
            state.name,
            state.timestamp.unwrap()
        );
        // Ensure we got a state reported by a registered collector, if not then we ignore it
        match self.server.contains_collector(state.name.clone()).await {
            true => {
                self.server
                    .report_collector_state(state.clone().into())
                    .await;
                Ok(TonicResponse::new(ReportStateResponse {
                    acknowledged: true,
                }))
            }
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

        if let None = query {
            return Ok(tonic::Response::new(QueryResponse { keys: vec![] }));
        }
        println!("[QUERY] Received a query request: {:?}", query);
        let query = query.unwrap();
        let mut uuids: Vec<LifelogFrameKey> = vec![];
        // NOTE: Right now we just return all uuids for a query, in the future actually parse the
        // query message and return the uuids that match
        let query = String::from("");
        let keys = self
            .server
            .process_query(query.clone())
            .await
            .map_err(|e| {
                tonic::Status::internal(format!("Failed to process query: {}", e.to_string()))
            })?;
        let proto_keys: Vec<lifelog_proto::LifelogDataKey> = keys
            .iter()
            .map(|key| lifelog_proto::LifelogDataKey {
                uuid: key.uuid.to_string(),
                origin: key.origin.get_table_name(),
            })
            .collect();
        Ok(tonic::Response::new(QueryResponse { keys: proto_keys }))
    }

    async fn get_state(
        &self,
        _request: tonic::Request<GetStateRequest>,
    ) -> Result<TonicResponse<GetSystemStateResponse>, TonicStatus> {
        println!("Received a get state request!");
        let state = self.server.get_state().await;
        //let proto_state = s
        Ok(TonicResponse::new(GetSystemStateResponse {
            state: Some(state.into()), // TODO: Replace this with the system state
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
            && !state
                .server_state
                .pending_actions
                .iter()
                .any(|a| matches!(a, &ServerActionType::SyncData))
        // TODO: Refactor so this happens once
        {
            // TODO: Add the specific data modality here
            ServerAction::SyncData("SELECT * FROM screen".to_string()) // TODO: Refactor so that we
                                                                       // can sync from all collectors
        } else if !state
            .server_state
            .pending_actions
            .iter()
            .any(|a| matches!(a, ServerActionType::TransformData))
        {
            ServerAction::TransformData(vec![])
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
        let collectors = self.registered_collectors.read().await;
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

    async fn process_query(&self, query: String) -> Result<Vec<LifelogFrameKey>, LifelogError> {
        let mut keys: Vec<LifelogFrameKey> = vec![];
        println!("asdfadsfasdf");
        // TODO: Refactor this so we add origins in a more intelligent way instead of checking them
        // every time
        let mut origins = self.origins.write().await;
        *origins = get_origins_from_db(&self.db)
            .await
            .expect("Failed to get origins from db");

        for origin in origins.iter() {
            println!("[PROCESS_QUERY]: Looking at origin {}", origin);
            let result = get_all_uuids_from_origin(&self.db, &origin).await;
            match result {
                Ok(uuids_from_origin) => {
                    keys.extend(uuids_from_origin.iter().map(|uuid| LifelogFrameKey {
                        uuid: *uuid,
                        origin: origin.clone(),
                    }));
                }
                Err(e) => {
                    return Err(LifelogError::Database(format!(
                        "Failed to get uuids from origin: {}",
                        e
                    )));
                }
            }
        }
        Ok(keys)
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
                {
                    self.state
                        .write()
                        .await
                        .server_state
                        .pending_actions
                        .push(ServerActionType::SyncData);
                }
                // TODO: Refactor so we actually use the query
                // Get the target data modalities(s) from the query
                let registered_collectors_clone = self.registered_collectors.clone();
                let db_connection = self.db.clone();
                let state_clone = self.state.clone();
                tokio::spawn(async move {
                    let mut collectors = registered_collectors_clone.write().await;
                    sync_data_with_collectors(
                        state.clone(),
                        &db_connection,
                        query,
                        &mut collectors,
                    )
                    .await;

                    // TODO: refactor, i dont think we should write lock the state here, diff
                    // function for estimating the state?
                    let mut state = state_clone.write().await;
                    state.server_state.timestamp_of_last_sync = Utc::now();
                    state.server_state.pending_actions.retain(|a| {
                        if let ServerActionType::SyncData = a {
                            return false;
                        }
                        true
                    });
                });

                // For now, assume we want to sync all data modalities

                // Ask the collectors to send data
            }
            ServerAction::TransformData(untransformed_data_keys) => {
                // TODO: Move this to policy
                println!("[TRANSFORM_DATA] Waiting for state write lock to be released");
                {
                    self.state
                        .write()
                        .await
                        .server_state
                        .pending_actions
                        .push(ServerActionType::TransformData); // TODO: Refactor this function s ow e don't hold the state write block
                }
                let state_clone = self.state.clone();
                let db_connection = self.db.clone();
                let transforms = self.transforms.clone().read().await.to_vec();
                println!("[TRANSFORM_DATA] starting thread");
                let res = tokio::spawn(async move {
                    let mut untransformed_data_keys: Vec<LifelogFrameKey> = vec![];
                    println!("[TRANSFORM_DATA]: Transforming data");
                    for transform in &transforms {
                        untransformed_data_keys.extend(
                            get_keys_in_source_not_in_destination(
                                &db_connection,
                                transform.source().clone(),
                                transform.destination().clone(),
                            )
                            .await,
                        );
                    }

                    transform_data(&db_connection, untransformed_data_keys, transforms).await;
                    // TODO: Refactor so not directly calling the .write on state
                    state_clone
                        .write()
                        .await
                        .server_state
                        .pending_actions
                        .retain(|a| {
                            if let ServerActionType::TransformData = a {
                                return false;
                            }
                            true
                        });
                });
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

// TODO: Complete this for every data type, make this into a MACRO
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ScreenFrameSurreal {
    //pub uuid: String,
    pub timestamp: surrealdb::sql::Datetime,
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

impl From<ScreenFrameSurreal> for ScreenFrame {
    fn from(frame: ScreenFrameSurreal) -> Self {
        Self {
            uuid: uuid::Uuid::from_u128(0),
            timestamp: frame.timestamp.into(),
            width: frame.width as u32,
            height: frame.height as u32,
            image_bytes: frame.image_bytes.into(),
            mime_type: frame.mime_type,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrowserFrameSurreal {
    pub timestamp: surrealdb::sql::Datetime,
    pub url: String,
    pub title: String,
    pub visit_count: i32,
}

impl From<BrowserFrame> for BrowserFrameSurreal {
    fn from(frame: BrowserFrame) -> Self {
        Self {
            timestamp: frame.timestamp.into(),
            url: frame.url,
            title: frame.title,
            visit_count: frame.visit_count as i32,
        }
    }
}

impl From<BrowserFrameSurreal> for BrowserFrame {
    fn from(frame: BrowserFrameSurreal) -> Self {
        Self {
            uuid: uuid::Uuid::from_u128(0),
            timestamp: frame.timestamp.into(),
            url: frame.url,
            title: frame.title,
            visit_count: frame.visit_count as u32,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OcrFrameSurreal {
    pub timestamp: surrealdb::sql::Datetime,
    pub text: String,
}

impl From<OcrFrame> for OcrFrameSurreal {
    fn from(frame: OcrFrame) -> Self {
        Self {
            timestamp: frame.timestamp.into(),
            text: frame.text,
        }
    }
}

impl From<OcrFrameSurreal> for OcrFrame {
    fn from(frame: OcrFrameSurreal) -> Self {
        Self {
            uuid: uuid::Uuid::from_u128(0),
            timestamp: frame.timestamp.into(),
            text: frame.text,
        }
    }
}

///// Convert `prost_types::Timestamp` → RFC-3339 string recognised by SurrealDB.
//fn ts_to_rfc3339(ts: &prost_types::Timestamp) -> String {
//    // Safety: prost Timestamp always contains valid seconds/nanos
//    let dt: DateTime<Utc> = chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32).unwrap();
//    dt.to_rfc3339()
//}
//
//fn build_time_filter(ranges: &[Timerange]) -> (String, Vec<(&'static str, Value)>) {
//    if ranges.is_empty() {
//        return ("true".into(), Vec::new());
//    }
//
//    let mut clauses = Vec::with_capacity(ranges.len());
//    let mut binds = Vec::with_capacity(ranges.len() * 2);
//
//    for (i, tr) in ranges.iter().enumerate() {
//        let start = ts_to_rfc3339(tr.start.as_ref().expect("missing start"));
//        let end = ts_to_rfc3339(tr.end.as_ref().expect("missing end"));
//
//        // (timestamp >= $s0 AND timestamp <= $e0)
//        clauses.push(format!("(timestamp >= $s{i} AND timestamp <= $e{i})"));
//
//        binds.push((Box::leak(format!("s{i}").into_boxed_str()), start.into()));
//        binds.push((Box::leak(format!("e{i}").into_boxed_str()), end.into()));
//    }
//
//    (clauses.join(" OR "), binds)
//}
//
///// Main helper – run the query and get the matching UUIDs.
/////
///// * `db`           – an *already authorised* Surreal handle
///// * `query`        – the incoming gRPC `Query`
/////
///// Returns a deduplicated vector of UUID strings.
//pub async fn query_uuids(
//    db: &Surreal<Client>,
//    query: Query,
//) -> Result<Vec<String>, surrealdb::Error> {
//    // Decide which tables we need to hit:
//    //   * search_sources drive the filtering
//    //   * if return_sources is empty, fall back to search_sources
//    let search_sources = if query.search_sources.is_empty() {
//        &query.return_sources
//    } else {
//        &query.search_sources
//    };
//
//    let return_sources = if query.return_sources.is_empty() {
//        search_sources
//    } else {
//        &query.return_sources
//    };
//
//    // Time filter ------------------------------------------------------------
//    let (time_filter, mut bindings) = build_time_filter(&query.time_ranges);
//
//    let browser_filter = true;
//    let browser_bind = None;
//
//    //// Optional browser-text filter ------------------------------------------
//    //let (browser_filter, browser_bind) = if query.browser_text.is_empty() {
//    //    ("true".into(), None)
//    //} else {
//    //    (
//    //        "(title CONTAINS $btxt OR url CONTAINS $btxt)".into(),
//    //        Some(("btxt", query.browser_text.clone().into())),
//    //    )
//    //};
//
//    if let Some(b) = browser_bind {
//        bindings.push(b);
//    }
//
//    // Collect UUIDs from all tables -----------------------------------------
//    let mut uuids = HashSet::<String>::new();
//
//    for table_str in return_sources {
//        // Our helper that parses "device:modality" → actual table name
//        let table = DataOrigin::from_string(table_str.clone()).get_table_name();
//
//        let sql = format!(
//            r#"
//            SELECT VALUE record::id(id)
//            FROM type::table($tbl)
//            WHERE ({time_filter}) AND ({browser_filter});
//            "#
//        );
//        println!("SQL: {}", sql);
//        panic!("SQL: {}", sql);
//
//        //// Run the query in one round-trip
//        //let mut resp = db
//        //    .query(sql)
//        //    .bind(("tbl", &table))
//        //    //.bind_many(bindings.clone())
//        //    .await?;
//
//        //let ids: Vec<String> = resp.take(0)?; // flat array because SELECT VALUE
//        //uuids.extend(ids);
//    }
//
//    Ok(uuids.into_iter().collect())
//}
//

async fn sync_data_with_collectors(
    state: SystemState,
    db: &Surreal<Client>,
    query: String,
    collectors: &mut Vec<RegisteredCollector>,
) -> Result<(), LifelogError> {
    for collector in collectors.iter_mut() {
        // TODO: Parallelize this
        println!("Syncing data with collector: {:?}", collector);
        // TODO: This code can fail here (notice the unwraps, I should handle it.
        let mut stream = collector
            .grpc_client
            .get_data(GetDataRequest { keys: vec![] })
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
        for chunk in data {
            // record id = random UUID
            let chunk = chunk.unwrap();

            // TODO: this can be automated with a macro
            match chunk {
                lifelog_proto::lifelog_data::Payload::Screenframe(c) => {
                    let data_origin = DataOrigin::new(
                        DataOriginType::DeviceId(mac.clone()),
                        DataModality::Screen,
                    );
                    let record = add_data_to_db::<ScreenFrame, ScreenFrameSurreal>(
                        &db,
                        c.into(),
                        &data_origin,
                    )
                    .await;
                }
                lifelog_proto::lifelog_data::Payload::Browserframe(c) => {
                    let data_origin = DataOrigin::new(
                        DataOriginType::DeviceId(mac.clone()),
                        DataModality::Browser,
                    );

                    let record = add_data_to_db::<BrowserFrame, BrowserFrameSurreal>(
                        &db,
                        c.into(),
                        &data_origin,
                    )
                    .await;
                }
                _ => unimplemented!(),
            };

            //db.create("audit_log")
            //  .content(json!({
            //      "ts": chrono::Utc::now(),
            //      "actor": msg.source_mac,
            //      "action": "ingest",
            //      "detail": { "table": table }
            //  }))
            //  .await?;
        }
    }
    Ok(())
}

async fn get_all_uuids_from_origin(
    db: &Surreal<Client>,
    data_origin: &DataOrigin,
) -> Result<Vec<Uuid>, surrealdb::Error> {
    let table = data_origin.get_table_name();
    let sql = format!("SELECT VALUE record::id(id) as uuid FROM `{table}`"); //FIX: Sql injection 🤡
    let uuids: Vec<String> = db
        .query(sql)
        .await
        .expect("Couldn't do the query")
        .take(0)
        .expect("We should only ever have one query");
    let uuids = uuids
        .into_iter()
        .map(|s| {
            let uuid = s.parse::<Uuid>().expect("Unable to go from string to uuid");
            uuid
        })
        .collect::<Vec<Uuid>>();
    Ok(uuids)
}

async fn get_keys_in_source_not_in_destination(
    db: &Surreal<Client>,
    source: DataOrigin,
    destination: DataOrigin,
) -> Vec<LifelogFrameKey> {
    // Get the record uuids from source
    let uuids_from_source = get_all_uuids_from_origin(&db, &source)
        .await
        .expect(format!("Unable to get uuids from source: {}", source).as_str());

    // Get the record uuids from destination
    let uuids_from_destination = get_all_uuids_from_origin(&db, &destination)
        .await
        .expect(format!("Unable to get uuids from destination: {}", destination).as_str());

    // Get the record uuids from source that are not in destination
    let uuids_in_source_not_in_destination: Vec<LifelogFrameKey> = uuids_from_source
        .iter()
        .filter(|uuid| !uuids_from_destination.contains(uuid))
        .cloned()
        .map(|uuid| {
            let key = LifelogFrameKey::new(uuid, source.clone());
            key
        })
        .collect();

    return uuids_in_source_not_in_destination;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum LifelogTransform {
    OcrTransform(OcrTransform),
}

impl LifelogTransform {
    fn source(&self) -> DataOrigin {
        match self {
            LifelogTransform::OcrTransform(transform) => transform.source(),
            _ => unimplemented!(),
        }
    }
    fn destination(&self) -> DataOrigin {
        match self {
            LifelogTransform::OcrTransform(transform) => transform.destination(),
            _ => unimplemented!(),
        }
    }
}

impl From<OcrTransform> for LifelogTransform {
    fn from(transform: OcrTransform) -> Self {
        Self::OcrTransform(transform)
    }
}

async fn transform_data(
    db: &Surreal<Client>,
    untransformed_data_keys: Vec<LifelogFrameKey>,
    transforms: Vec<LifelogTransform>,
) {
    for key in untransformed_data_keys.iter() {
        let data_to_transform: LifelogData = get_data_by_key(db, key)
            .await
            .expect(format!("Unable to get data by key: {}", key).as_str());
        println!(
            "[TRANSFORM_DATA]: Transforming data: <{}>:<{}>",
            key.origin.get_table_name(),
            key.uuid
        );
        for transform in transforms.iter() {
            // Check what transforms apply to these keys
            let transformed_data: Option<LifelogData> = match transform {
                LifelogTransform::OcrTransform(transform) => {
                    if key.origin == transform.source() {
                        let mut result = transform
                                .apply(data_to_transform.clone().try_into().expect("Data source is not a lifelog image!"))
                                .expect(format!("This should never error because the origins {} {} are the same", key.origin, transform.source()).as_str());

                        result.uuid = key.uuid; // NOTE: THIS IS IMPORTANT. THIS NEEDS TO BE FIXED
                                                // WITH A CODE REFACTOR
                        Some(result.into())
                    } else {
                        None
                    }
                }
                _ => unimplemented!(),
            };
            let transformed_data = match transformed_data {
                Some(data) => data,
                None => continue,
            };
            match transformed_data {
                LifelogData::OcrFrame(ocr_frame) => {
                    add_data_to_db::<OcrFrame, OcrFrameSurreal>(
                        &db,
                        ocr_frame,
                        &transform.destination(),
                    )
                    .await
                    .unwrap();
                }
                _ => unimplemented!(),
            }
        }
    }
}

async fn get_data_by_key(
    db: &Surreal<Client>,
    key: &LifelogFrameKey,
) -> Result<LifelogData, anyhow::Error> {
    match key.origin.modality {
        DataModality::Screen => {
            let row: Option<ScreenFrameSurreal> = db
                .select((key.origin.get_table_name(), key.uuid.to_string()))
                .await?;
            let mut screen_frame: ScreenFrame = row
                .expect(
                    format!(
                        "Unabled to find record <{}>:<{}>",
                        key.origin.get_table_name(),
                        key.uuid
                    )
                    .as_str(),
                )
                .into();
            screen_frame.uuid = key.uuid; //NOTE: This is important. This needs to be fixed with a code refactor
            Ok(screen_frame.into())
        }
        DataModality::Ocr => {
            let row: Option<OcrFrameSurreal> = db
                .select((key.origin.get_table_name(), key.uuid.to_string()))
                .await?;
            let mut ocr_frame: OcrFrame = row
                .expect(
                    format!(
                        "Unabled to find record <{}>:<{}>",
                        key.origin.get_table_name(),
                        key.uuid
                    )
                    .as_str(),
                )
                .into();
            ocr_frame.uuid = key.uuid; //NOTE: This is important. This needs to be fixed with a code refactor
            Ok(ocr_frame.into())
        }
        DataModality::Browser => {
            let row: Option<BrowserFrameSurreal> = db
                .select((key.origin.get_table_name(), key.uuid.to_string()))
                .await?;
            let mut browser_frame: BrowserFrame = row
                .expect(
                    format!(
                        "Unabled to find record <{}>:<{}>",
                        key.origin.get_table_name(),
                        key.uuid
                    )
                    .as_str(),
                )
                .into();
            browser_frame.uuid = key.uuid; //NOTE: This is important. This needs to be fixed with a code refactor
            Ok(browser_frame.into())
        }
        _ => unimplemented!(),
    }
}

async fn add_data_to_db<LifelogType, SurrealType>(
    db: &Surreal<Client>,
    data: LifelogType,
    data_origin: &DataOrigin,
) -> surrealdb::Result<SurrealType>
where
    LifelogType: Into<SurrealType> + DataType,
    SurrealType: Serialize + DeserializeOwned + 'static,
{
    let uuid = data.uuid();
    let table = format!("{}", data_origin.get_table_name());
    ensure_table(&db, &data_origin).await.unwrap();
    let data: SurrealType = data.into();
    let record: Result<Option<SurrealType>, Error> = db
        .create((table.clone(), uuid.to_string()))
        .content(data)
        .await;
    println!("[SURREAL]: Created <{}:{}>", table, uuid);
    match record {
        Err(e) => {
            eprintln!("{}", e);
            Err(e)
        }
        Ok(record) => {
            let record = record.expect(format!("Unable to create row in table {}", table).as_str());
            Ok(record)
        }
    }
}

// TODO: This should be automated
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum LifelogData {
    ScreenFrame(ScreenFrame),
    BrowserFrame(BrowserFrame),
    OcrFrame(OcrFrame),
}

impl From<LifelogData> for lifelog_proto::LifelogData {
    fn from(data: LifelogData) -> Self {
        match data {
            LifelogData::ScreenFrame(frame) => lifelog_proto::LifelogData {
                payload: Some(lifelog_proto::lifelog_data::Payload::Screenframe(
                    frame.into(),
                )),
            },
            LifelogData::BrowserFrame(frame) => lifelog_proto::LifelogData {
                payload: Some(lifelog_proto::lifelog_data::Payload::Browserframe(
                    frame.into(),
                )),
            },
            LifelogData::OcrFrame(frame) => lifelog_proto::LifelogData {
                payload: Some(lifelog_proto::lifelog_data::Payload::Ocrframe(frame.into())),
            },
        }
    }
}

impl From<lifelog_proto::LifelogData> for LifelogData {
    fn from(data: lifelog_proto::LifelogData) -> Self {
        match data.payload {
            Some(lifelog_proto::lifelog_data::Payload::Screenframe(frame)) => {
                LifelogData::ScreenFrame(frame.into())
            }
            Some(lifelog_proto::lifelog_data::Payload::Browserframe(frame)) => {
                LifelogData::BrowserFrame(frame.into())
            }
            Some(lifelog_proto::lifelog_data::Payload::Ocrframe(frame)) => {
                LifelogData::OcrFrame(frame.into())
            }
            _ => unimplemented!(),
        }
    }
}

impl TryFrom<LifelogData> for LifelogImage {
    type Error = anyhow::Error;
    fn try_from(v: LifelogData) -> Result<Self, Self::Error> {
        match v {
            LifelogData::ScreenFrame(frame) => Ok(frame.into()),
            _ => Err(anyhow::anyhow!("Cannot convert to LifelogImage")),
        }
    }
}

impl From<ScreenFrame> for LifelogData {
    fn from(v: ScreenFrame) -> Self {
        Self::ScreenFrame(v)
    }
}
impl From<BrowserFrame> for LifelogData {
    fn from(v: BrowserFrame) -> Self {
        Self::BrowserFrame(v)
    }
}

impl From<OcrFrame> for LifelogData {
    fn from(v: OcrFrame) -> Self {
        Self::OcrFrame(v)
    }
}

pub async fn get_tables(db: &Surreal<Client>) -> Result<Vec<String>, LifelogError> {
    #[derive(serde::Deserialize)]
    struct Info {
        tables: std::collections::HashMap<String, serde_json::Value>,
    }

    let mut resp = db
        .query("INFO FOR DB")
        .await
        .map_err(|e| LifelogError::Database(format!("{}", e)))?;

    let info: Option<Info> = resp
        .take(0)
        .map_err(|e| LifelogError::Database(format!("{}", e)))?;
    let info = info.ok_or_else(|| LifelogError::Database("INFO FOR DB failed!!!".to_string()))?;
    Ok(info.tables.keys().cloned().collect())
}

async fn get_origins_from_db(db: &Surreal<Client>) -> Result<Vec<DataOrigin>, LifelogError> {
    let tables = get_tables(&db).await?;
    let origins = tables
        .iter()
        .map(|table| {
            let origin = DataOrigin::tryfrom_string(table.clone());
            origin
        })
        .filter(Result::is_ok)
        .map(|origin| origin.expect("this should never happen"))
        .collect::<Vec<DataOrigin>>();

    Ok(origins)
}
