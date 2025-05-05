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
use surrealdb::engine::local::Db;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tonic::{Request as TonicRequest, Response as TonicResponse, Status as TonicStatus};

use config::CollectorConfig;
use proto::lifelog_server_service_server::LifelogServerService;
use proto::{
    GetDataRequest, GetDataResponse, GetStateRequest, GetSystemConfigRequest,
    GetSystemConfigResponse, GetSystemStateResponse, QueryRequest, QueryResponse,
    RegisterCollectorRequest, RegisterCollectorResponse, ReportStateRequest, ReportStateResponse,
    SetSystemConfigRequest, SetSystemConfigResponse,
};
use tokio::sync::oneshot;

pub mod proto {
    tonic::include_proto!("lifelog");
    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("lifelog_descriptor");
}

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

type Query = String;

// TODO: Automatically generate the RPCs for this code so that every action is it's own RPC,
// automatically generate the code for every RPC as they are the exact same code!
#[derive(Debug)]
pub enum ServerCommand {
    // These are commands to the servers, each of them can result in [0-n] actions. If it is
    // something that can be immediately resolved (such as registering a collector) then it will
    // result in no actions done,
    RegisterCollector(RegisterCollectorRequest),
    GetConfig(GetSystemConfigRequest),
    SetConfig(SetSystemConfigRequest),
    GetData(
        GetDataRequest,
        oneshot::Sender<Result<GetDataResponse, ServerError>>,
    ),
    Query(QueryRequest),
    ReportState(ReportStateRequest),
    GetState(GetStateRequest),
}

// Convert server commands to action

impl ServerCommand {
    pub fn to_actions(&self) -> Vec<ServerAction> {
        match self {
            ServerCommand::RegisterCollector(request) => {
                vec![]
            }
            ServerCommand::GetConfig(_) => {
                vec![]
            }
            ServerCommand::SetConfig(_) => vec![],
            ServerCommand::Query(request) => vec![ServerAction::Query(request.clone())],
            _ => {
                panic!("Command {:?} not implemented yet", self);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActorConfig;

// TODO: Add all actions to a swervice so any program can tell the server to do anything

#[derive(Debug, Clone)]
pub enum ServerAction {
    Sleep(tokio::time::Duration), // Sleep for a certain amount of time)
    Query(proto::QueryRequest),
    GetData(proto::GetDataRequest), // TODO: Wouldn't it be cool if the system could specify exactly what data
    // it wanted from the collector so when it has a query it doesn't need to process everything?
    SyncData(Query),
    HealthCheck,
    ReceiveData(Vec<Uuid>),
    CompressData(Vec<Uuid>),
    TransformData(Vec<Uuid>),
    RegisterActor(ActorConfig),
}

// TDOO: ADD A CHANNEL FOR COMMS
#[derive(Clone, Debug)]
pub struct Server {
    db: Surreal<Db>,
    host: String,
    port: u16,
    state: Arc<RwLock<SystemState>>,
    register_collectors: Arc<RwLock<Vec<RegisteredCollector>>>,
    register_interfaces: Arc<RwLock<Vec<RegisteredInterface>>>,
    policy: Arc<RwLock<ServerPolicy>>,

    cmd_tx: mpsc::Sender<ServerCommand>,
    pending_commands: Arc<RwLock<Vec<ServerCommand>>>,
}

const SERVER_COMMAND_CHANNEL_BUFFER_SIZE: usize = 100;

impl Server {
    pub async fn new(config: &ServerConfig) -> Result<Self, ServerError> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("lifelog")
            .use_db(config.database_name.clone())
            .await?;

        let (cmd_tx, mut cmd_rx) =
            mpsc::channel::<ServerCommand>(SERVER_COMMAND_CHANNEL_BUFFER_SIZE);

        tokio::task::spawn(async move {
            while let Some(command) = cmd_rx.recv().await {
                println!("Got Command {:?}", command);
                match command {
                    ServerCommand::GetData(req, tx) => {
                        tx.send(Ok(GetDataResponse::default()));
                    }
                    _ => todo!(),
                }
            }
        });

        let state = SystemState {
            server_state: ServerState {
                name: config.server_name.clone(),
                timestamp: Utc::now(),
                ..Default::default()
            },
            ..Default::default()
        };
        let state = Arc::new(RwLock::new(state));

        let policy = Arc::new(RwLock::new(ServerPolicy {
            config: ServerPolicyConfig::default(),
        }));

        Ok(Self {
            db,
            host: config.host.clone(),
            port: config.port,
            state,
            register_collectors: Arc::new(RwLock::new(vec![])),
            register_interfaces: Arc::new(RwLock::new(vec![])),
            policy,
            pending_commands: Arc::new(RwLock::new(vec![])),
            cmd_tx,
        })
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

        // 1) make a oneshot pair
        let (tx, rx) = oneshot::channel::<Result<GetDataResponse, ServerError>>();

        // 2) send the command + responder out to your scheduler
        self.cmd_tx
            .send(ServerCommand::GetData(req, tx))
            .await
            .map_err(|_| TonicStatus::internal("scheduler down"))?;

        // 3) WAIT here for the scheduler to run it and send you back the response
        let result = rx
            .await
            .map_err(|_| TonicStatus::internal("worker dropped without answering"))
            .map_err(|e| TonicStatus::internal(e.to_string()))?;

        // 4) turn it into a gRPC response
        match result {
            Ok(resp) => Ok(TonicResponse::new(resp)),
            Err(stat) => Err(TonicStatus::internal(stat.to_string())),
        }
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
            uuids: vec![proto::Uuid {
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
            state: Some(proto::ServerState::default()), // TODO: replace this with an .into for the
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

        let action = ServerAction::Sleep(tokio::time::Duration::from_secs(1));

        action
    }
}

impl Server {
    /// This function will be run upon startup, it will handle the server's main loop of doing
    /// actions
    pub async fn policy_loop(&self) {
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
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    fn get_policy(&self) -> Arc<RwLock<ServerPolicy>> {
        // Get the policy from the config
        return self.policy.clone();
    }

    async fn get_state(&self) -> SystemState {
        // Estimate the state in this function

        return self.state.read().await.clone();
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
                println!("sleeping");
            }
            _ => todo!(),
        }
    }
}
