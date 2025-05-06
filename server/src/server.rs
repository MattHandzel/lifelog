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

use config::CollectorConfig;
use lifelog_proto::lifelog_server_service_server::LifelogServerService;
use lifelog_proto::{
    GetDataRequest, GetDataResponse, GetStateRequest, GetSystemConfigRequest,
    GetSystemConfigResponse, GetSystemStateResponse, QueryRequest, QueryResponse,
    RegisterCollectorRequest, RegisterCollectorResponse, ReportStateRequest, ReportStateResponse,
    SetSystemConfigRequest, SetSystemConfigResponse,
};
use tokio::sync::oneshot;

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

// TDOO: ADD A CHANNEL FOR COMMS
#[derive(Debug)]
pub struct Server<'a> {
    db: Surreal<Client>,
    host: String,
    port: u16,
    state: Arc<RwLock<SystemState>>,
    register_collectors: Arc<RwLock<Vec<RegisteredCollector>>>,
    register_interfaces: Arc<RwLock<Vec<RegisteredInterface>>>,
    policy: Arc<RwLock<ServerPolicy>>,

    pending_commands: Arc<RwLock<Vec<ServerCommand>>>,
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
                timestamp: Utc::now(),
                ..Default::default()
            },
            ..Default::default()
        };
        let state = Arc::new(RwLock::new(state));

        let policy = Arc::new(RwLock::new(ServerPolicy {
            config: ServerPolicyConfig::default(),
        }));

        let (cmd_tx, mut cmd_rx) = mpsc::channel(SERVER_COMMAND_CHANNEL_BUFFER_SIZE);

        let s = Self {
            db,
            host: config.host.clone(),
            port: config.port as u16,
            state,
            register_collectors: Arc::new(RwLock::new(vec![])),
            register_interfaces: Arc::new(RwLock::new(vec![])),
            policy,
            pending_commands: Arc::new(RwLock::new(vec![])),
            cmd_tx: cmd_tx,
            cmd_rx: cmd_rx,
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

        // 1) make a oneshot pair
        let (tx, rx) = oneshot::channel::<Result<GetDataResponse, ServerError>>();

        // 2) send the command + responder out to your scheduler
        self.cmd_tx
            .as_ref()
            .expect("Scheduler not started")
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

        let action = ServerAction::Sleep(tokio::time::Duration::from_secs(1));

        action
    }
}

impl Server {
    /// This function will be run upon startup, it will handle the server's main loop of doing
    /// actions
    pub async fn r#loop(&self) -> ! {
        // Set up command tokio channel
        let pending_commands = self.pending_commands.clone();
        tokio::task::spawn(async move {
            while let Some(command) = &self.cmd_rx.recv().await {
                println!("Got Command {:?}", command);
                match command {
                    ServerCommand::GetData(ref req, ref tx) => {
                        pending_commands.write().await.push(command);
                    }
                    _ => todo!(),
                }
            }
        });
        let policy = self.get_policy();

        loop {
            let state = self.get_state().await;

            let mut pending_commands_vec = self.pending_commands.write().await;
            let action = if pending_commands_vec.len() > 0 {
                // TODO: REFACTOR TO SUPPORT MULTI-ACTION COMMANDS
                let command = pending_commands_vec.pop().unwrap();
                command.to_actions()[0].clone()
            } else {
                policy.read().await.get_action(&state)
            };

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
            }
            _ => todo!(),
        }
    }
}
