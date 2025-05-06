use crate::policy::*;
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

use sysinfo::System;

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

    pub async fn register_collector(&self, collector: RegisteredCollector) {
        println!("Trying to register collector: {:?}", collector);
        let mut server = self.server.write().await;
        server.register_collectors.write().await.push(collector);
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
                println!("Trying to connect to endpoint: {:?}", endpoint);
                let endpoint = endpoint.connect_timeout(time::Duration::from_secs(10));

                let channel = endpoint.connect().await.map_err(|e| {
                    TonicStatus::internal(format!("Failed to connect to endpoint: {}", e))
                })?;
                let client = CollectorServiceClient::new(channel);

                let collector = RegisteredCollector {
                    id: collector_config.id.clone(),
                    address: collector_ip.to_string(),
                    grpc_client: client.clone(),
                };
                println!("Collector: {:?}", collector);
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
        Ok(TonicResponse::new(ReportStateResponse {
            acknowledged: true,
        }))
    }

    async fn query(
        &self,
        request: tonic::Request<QueryRequest>,
    ) -> Result<TonicResponse<QueryResponse>, TonicStatus> {
        let inner = request.into_inner();
        println!("Received a query request! {:?}", inner.query);
        Ok(TonicResponse::new(QueryResponse {
            uuids: vec![lifelog_proto::Uuid {
                uuid: "2b6e8293-1300-4318-9196-f8fed905b499".to_string(),
            }],
        }))
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

        // TODO: Look at the collector states and when a collector is more than 60 seconds out of
        // sync ask it for more data
        let action = if state.server_state.timestamp.timestamp() % 10 == 0 {
            // TODO: Add the specific data modality here
            ServerAction::SyncData("SELECT * FROM screen".to_string())
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
                                                            //state.server_state.threads = 0.0; // TODO: Get the real number of threads
                                                            //                                  // TODO: There is a race condition here, someone can grab the lock before we can grab it
            state.clone()
        }
    }

    async fn add_audit_log(&self, action: &ServerAction) {
        println!("Adding audit log for action: {:?}", action);
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
                // Get the target data modalities(s) from the query
                let collectors = self.register_collectors.read().await;
                println!("Syncing data with collectors: {:?}", collectors);
                //for collector in   {
                //    println!("Syncing data with collector: {:?}", collector);
                //    let data = collector
                //        .grpc_client
                //        .get_data(GetDataRequest {
                //            uuids: vec![query.clone().into()],
                //        })
                //        .await
                //        .unwrap();
                //    println!("Data: {:?}", data);
                //}
                //
                // For now, assume we want to sync all data modalities

                // Ask the collectors to send data
            }
            _ => todo!(),
        }
    }
}
