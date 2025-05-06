use crate::policy::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use config::ServerConfig;
use config::ServerPolicyConfig;
use dashmap::DashMap;
use lifelog_core::*;
use lifelog_types::CollectorState;
use lifelog_types::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tonic::{Request as TonicRequest, Response as TonicResponse, Status as TonicStatus};

use strum::IntoEnumIterator;

use config::CollectorConfig;
use data_modalities::screen::ScreenFrame;
use lifelog_proto::lifelog_server_service_server::LifelogServerService;
use lifelog_proto::LifelogData;
use lifelog_proto::{
    GetDataRequest, GetDataResponse, GetStateRequest, GetSystemConfigRequest,
    GetSystemConfigResponse, GetSystemStateResponse, QueryRequest, QueryResponse,
    RegisterCollectorRequest, RegisterCollectorResponse, ReportStateRequest, ReportStateResponse,
    SetSystemConfigRequest, SetSystemConfigResponse,
};
use lifelog_types::DataModality;
use tokio::sync::oneshot;

use once_cell::sync::Lazy;

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

#[tonic::async_trait]
impl LifelogServerService for Server {
    async fn register_collector(
        &self,
        request: TonicRequest<RegisterCollectorRequest>,
    ) -> Result<TonicResponse<RegisterCollectorResponse>, TonicStatus> {
        let inner = request.into_inner();
        Ok(TonicResponse::new(RegisterCollectorResponse {
            success: true,
            session_id: chrono::Utc::now().timestamp_subsec_nanos() as u64
                + chrono::Utc::now().timestamp() as u64,
        }))
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
        let state = self.state.read().await.clone();
        Ok(TonicResponse::new(GetSystemStateResponse {
            state: Some(ServerState::default().into()), // TODO: replace this with an .into for the
                                                        // server state
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
        println!("{:?}", state.server_state.timestamp.timestamp());
        let action = if state.server_state.timestamp.timestamp() % 10 == 0 {
            // TODO: Add the specific data modality here
            ServerAction::SyncData("SELECT * FROM screen".to_string())
        } else {
            ServerAction::Sleep(tokio::time::Duration::from_millis(100))
        };

        action
    }
}

impl Server {
    /// This function will be run upon startup, it will handle the server's main loop of doing
    /// actions
    pub async fn r#loop(&self) -> ! {
        // Set up command tokio channel
        let policy = self.get_policy();

        loop {
            let state = self.get_state().await;
            let action = policy.read().await.get_action(&state);

            // Perform the action
            // TODO: Czy mam problem że akcje byś mogły trwać długo? Jak to rozwiązać

            // TODO: dodaj parallelizm do tego, żeby nie czekać na zakończenie akcji
            //tokio::task::spawn(async move {
            //Add to audit log
            self.add_audit_log(&action).await;
            self.do_action(action, state).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // TODO: Remove
        }
    }

    fn get_policy(&self) -> Arc<RwLock<ServerPolicy>> {
        // Get the policy from the config
        return self.policy.clone();
    }

    async fn get_state(&self) -> SystemState {
        // Estimate the state in this function
        let mut state = self.state.write().await;
        state.server_state.timestamp = Utc::now();
        state.server_state.cpu_usage = 0.0; // TODO: Get the real CPU usage
        state.server_state.memory_usage = 0.0; // TODO: Get the real memory usage
        state.server_state.threads = 0.0; // TODO: Get the real number of threads
                                          // TODO: There is a race condition here, someone can grab the lock before we can grab it
        state.clone()
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

                // For now, assume we want to sync all data modalities

                // Ask the collectors to send data
            }
            _ => todo!(),
        }
    }
}
