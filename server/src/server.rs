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
use std::sync::Arc;
use surrealdb::engine::local::Db;
use surrealdb::engine::local::Mem;
use surrealdb::Surreal;
use thiserror::Error;
use tokio::sync::RwLock;
use tonic::{Response as TonicResponse, Status as TonicStatus};

use config::Config;
use proto::lifelog_server_service_server::LifelogServerService;
use proto::{
    GetConfigRequest, GetConfigResponse, GetDataRequest, GetDataResponse, GetStateRequest,
    GetStateResponse, RegisterCollectorRequest, RegisterCollectorResponse, ReportStateRequest,
    ReportStateResponse, SetConfigRequest, SetConfigResponse,
};

pub mod proto {
    tonic::include_proto!("lifelog");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
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
}

#[derive(Clone, Debug)]
pub struct Server {
    db: Surreal<Db>,
    host: String,
    port: u16,
    state: Arc<RwLock<SystemState>>,
    register_collectors: Arc<RwLock<Vec<RegisteredCollector>>>,
    register_interfaces: Arc<RwLock<Vec<RegisteredInterface>>>,
    policy: Arc<RwLock<ServerPolicy>>,
}

impl Server {
    pub async fn new(config: &ServerConfig) -> Result<Self, ServerError> {
        let db = Surreal::new::<Mem>(()).await?;
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

        Ok(Self {
            db,
            host: config.host.clone(),
            port: config.port,
            state,
            register_collectors: Arc::new(RwLock::new(vec![])),
            register_interfaces: Arc::new(RwLock::new(vec![])),
            policy,
        })
    }
}

#[tonic::async_trait]
impl LifelogServerService for Server {
    async fn register_collector(
        &self,
        request: tonic::Request<RegisterCollectorRequest>,
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
        _request: tonic::Request<GetConfigRequest>,
    ) -> Result<TonicResponse<GetConfigResponse>, TonicStatus> {
        println!("Received a get config request!");
        Ok(TonicResponse::new(GetConfigResponse::default()))
    }

    async fn set_config(
        &self,
        _request: tonic::Request<SetConfigRequest>,
    ) -> Result<TonicResponse<SetConfigResponse>, TonicStatus> {
        println!("Received a set config request!");
        Ok(TonicResponse::new(SetConfigResponse::default()))
    }

    async fn get_data(
        &self,
        _request: tonic::Request<GetDataRequest>,
    ) -> Result<TonicResponse<GetDataResponse>, TonicStatus> {
        println!("Received a get data request!");
        Ok(TonicResponse::new(GetDataResponse::default()))
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
}

type Query = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerAction {
    Sleep(tokio::time::Duration), // Sleep for a certain amount of time)
    Query(Query),
    RequestData(Vec<Uuid>),
    CompressData(Vec<Uuid>),
    TransformData(Vec<Uuid>),
    BackupData,
    CreateBackup,
    SendData,
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
    async fn policy_loop(&self) {
        let policy = self.get_policy();

        loop {
            let state = self.get_state().await;
            let action = policy.read().await.get_action(&state);

            // Perform the action
            // TODO: Is there a problem that the do action might take a long time to complete? How
            // to deal with that

            // TODO: Add parallelization/multithreading
            //tokio::task::spawn(async move {
            //Add to audit log
            self.add_audit_log(&action);
            self.do_action(action, state);
            //});
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
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

    async fn do_action(&self, action: ServerAction, state: SystemState) {
        // Perform the action
        match action {
            ServerAction::Sleep(duration) => {
                // Sleep for a certain amount of time
                tokio::time::sleep(duration).await;
            }
            _ => {
                println!("Action not supported yet");
            }
        }
    }
}
