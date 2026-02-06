use crate::policy::*;
use anyhow;
use chrono::Utc;
use config::ServerPolicyConfig;
use config::{CollectorConfig, ServerConfig, SystemConfig};
use data_modalities::*;
use lifelog_types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::RwLock;
use utils::cas::FsCas;

use crate::db::get_origins_from_db;
use crate::query::{get_all_uuids_from_origin, get_data_by_key};
use crate::sync::{get_keys_in_source_not_in_destination, sync_data_with_collectors};
use crate::transform::{transform_data, LifelogTransform};

// Re-export for external consumers (main.rs, tests)
pub use crate::grpc_service::GRPCServerLifelogServerService;

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
        let server_lock = self.server.read().await; // Acquire read lock on Server to access registered_collectors
        let mut collectors_vec = server_lock.registered_collectors.write().await; // Acquire write lock on the Vec

        if let Some(existing_collector) = collectors_vec.iter_mut().find(|c| c.id == collector.id) {
            // Collector already exists, update it
            tracing::info!("Updating existing collector: {:?}", collector.id);
            *existing_collector = collector;
        } else {
            // Collector does not exist, add it
            tracing::info!("Adding new collector: {:?}", collector.id);
            collectors_vec.push(collector);
        }
    }

    pub async fn contains_collector(&self, collector_name: String) -> bool {
        let server = self.server.read().await;
        server.contains_collector(collector_name).await
    }

    pub async fn report_collector_state(&self, state: CollectorState) -> Result<(), LifelogError> {
        let server = self.server.write().await;
        server.report_collector_state(state).await
    }

    pub async fn process_query(&self, query: String) -> Result<Vec<LifelogFrameKey>, LifelogError> {
        tracing::debug!(query = %query, "Requesting server lock for process_query");
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
    pub(crate) db: Surreal<Client>,
    state: Arc<RwLock<SystemState>>,
    pub(crate) registered_collectors: Arc<RwLock<Vec<RegisteredCollector>>>,
    policy: Arc<RwLock<ServerPolicy>>,
    origins: Arc<RwLock<Vec<DataOrigin>>>,
    transforms: Arc<RwLock<Vec<LifelogTransform>>>, // TODO: These should be registered transforms
    config: ServerConfig,
    pub(crate) cas: FsCas,
}

static SYS: OnceLock<Mutex<sysinfo::System>> = OnceLock::new();

impl Server {
    pub async fn new(config: &ServerConfig) -> Result<Self, LifelogError> {
        // Validate config before doing anything
        config.validate()?;

        let db = Surreal::new::<Ws>(&config.database_endpoint)
            .await
            .map_err(|e| {
                LifelogError::Database(format!(
                    "Could not connect to database at {}: {}",
                    config.database_endpoint, e
                ))
            })?;
        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .map_err(|e| LifelogError::Database(format!("{}", e)))?;

        db.use_ns("lifelog")
            .use_db(config.database_name.clone())
            .await
            .map_err(|e| LifelogError::Database(format!("{}", e)))?;

        let state = SystemState {
            server_state: Some(ServerState {
                name: config.server_name.clone(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let state = Arc::new(RwLock::new(state));

        let policy = Arc::new(RwLock::new(ServerPolicy {
            config: ServerPolicyConfig::default(),
        }));
        SYS.get_or_init(|| Mutex::new(sysinfo::System::new_all()));

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

        // Run startup schema migrations (ensures all tables + indexes exist)
        crate::schema::run_startup_migrations(&db).await?;

        let origins_vec = get_origins_from_db(&db).await?;

        let s = Self {
            db,
            state,
            registered_collectors: Arc::new(RwLock::new(vec![])),
            policy,
            transforms: Arc::new(RwLock::new(vec![ocr_transform.into()])),
            origins: Arc::new(RwLock::new(origins_vec)),
            config: config.clone(),
            cas: FsCas::new(config.cas_path.clone()),
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
                .ok_or_else(|| {
                    LifelogError::Other(anyhow::anyhow!(
                        "Collector {} returned no config",
                        collector.id
                    ))
                })?
                .into();
            collector_configs.insert(collector.id.clone(), config);
        }
        let config = SystemConfig {
            server: Some(self.config.clone()),
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
            let data: LifelogData = get_data_by_key(&self.db, key).await.map_err(|e| {
                LifelogError::Database(format!("Unable to get data by key {}: {}", key, e))
            })?;

            datas.push(data);
        }
        Ok(datas)
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
        let Some(ss) = state.server_state.as_ref() else {
            return ServerAction::Sleep(tokio::time::Duration::from_millis(100));
        };

        let t_now = ss
            .timestamp
            .as_ref()
            .map(|t| {
                chrono::DateTime::<Utc>::from_timestamp(t.seconds, t.nanos as u32)
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let t_last = ss
            .timestamp_of_last_sync
            .as_ref()
            .map(|t| {
                chrono::DateTime::<Utc>::from_timestamp(t.seconds, t.nanos as u32)
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let action = if (t_now - t_last).num_seconds() as f64
            >= (self.config.collector_sync_interval as f64)
            && !ss
                .pending_actions
                .contains(&(ServerActionType::SyncData as i32))
        {
            ServerAction::SyncData("SELECT * FROM screen".to_string())
        } else if !ss
            .pending_actions
            .contains(&(ServerActionType::TransformData as i32))
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
        tracing::debug!(collector = %collector_name, "Checking if collector is registered");
        for collector in collectors.iter() {
            if collector.id == collector_name {
                return true;
            }
        }
        false
    }

    async fn get_state(&self) -> SystemState {
        let (cpu_usage, memory_usage) = {
            let mut sys = SYS
                .get()
                .expect("SYS must be initialized in Server::new()")
                .lock()
                .expect("SYS mutex poisoned");
            sys.refresh_all();
            let cpu = sys.global_cpu_usage() / 100.0;
            let total = sys.total_memory() as f32;
            let used = sys.used_memory() as f32;
            let mem = if total > 0.0 { used / total } else { 0.0 };
            (cpu, mem)
        };

        let mut state = self.state.write().await;
        if let Some(ss) = state.server_state.as_mut() {
            ss.cpu_usage = cpu_usage;
            ss.timestamp = Some(Utc::now().into());
            ss.memory_usage = memory_usage;
        }
        state.clone()
    }

    async fn process_query(&self, query: String) -> Result<Vec<LifelogFrameKey>, LifelogError> {
        tracing::debug!(query = %query, "Entered process_query");
        let mut keys: Vec<LifelogFrameKey> = vec![];

        let mut origins = self.origins.write().await;

        match get_origins_from_db(&self.db).await {
            Ok(db_origins) => {
                tracing::debug!(count = db_origins.len(), "Refreshed origins from DB");
                *origins = db_origins;
            }
            Err(e) => {
                tracing::error!(error = %e, "Failed to get origins from DB");
                return Err(LifelogError::Other(anyhow::anyhow!(
                    "Failed to refresh origins from DB: {}",
                    e
                )));
            }
        }

        for origin in origins.iter() {
            tracing::trace!(origin = %origin, "Querying UUIDs for origin");
            match get_all_uuids_from_origin(&self.db, origin).await {
                Ok(uuids_from_origin) => {
                    keys.extend(uuids_from_origin.iter().map(|uuid| LifelogFrameKey {
                        uuid: *uuid,
                        origin: origin.clone(),
                    }));
                }
                Err(e) => {
                    tracing::warn!(origin = %origin, error = %e, "Failed to get UUIDs from origin");
                }
            }
        }
        tracing::debug!(count = keys.len(), "Process query complete");
        Ok(keys)
    }

    async fn add_audit_log(&self, _action: &ServerAction) {
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
                    if let Some(ss) = self.state.write().await.server_state.as_mut() {
                        ss.pending_actions.push(ServerActionType::SyncData as i32);
                    }
                }
                // TODO: Refactor so we actually use the query
                // Get the target data modalities(s) from the query
                let registered_collectors_clone = self.registered_collectors.clone();
                let db_connection = self.db.clone();
                let state_clone = self.state.clone();
                let _query = query;
                tokio::spawn(async move {
                    let mut collectors = registered_collectors_clone.write().await;
                    let _ = sync_data_with_collectors(
                        state.clone(),
                        &db_connection,
                        _query,
                        &mut collectors,
                    )
                    .await;

                    // TODO: refactor, i dont think we should write lock the state here, diff
                    // function for estimating the state?
                    let mut state = state_clone.write().await;
                    if let Some(ss) = state.server_state.as_mut() {
                        ss.timestamp_of_last_sync = Some(Utc::now().into());
                        ss.pending_actions
                            .retain(|&a| a != ServerActionType::SyncData as i32);
                    }
                });
            }
            ServerAction::TransformData(_untransformed_data_keys) => {
                {
                    if let Some(ss) = self.state.write().await.server_state.as_mut() {
                        ss.pending_actions
                            .push(ServerActionType::TransformData as i32);
                    }
                }
                let state_clone = self.state.clone();
                let db_connection = self.db.clone();
                let transforms = self.transforms.clone().read().await.to_vec();
                let _res = tokio::spawn(async move {
                    let mut untransformed_data_keys: Vec<LifelogFrameKey> = vec![];
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
                    if let Some(ss) = state_clone.write().await.server_state.as_mut() {
                        ss.pending_actions
                            .retain(|&a| a != ServerActionType::TransformData as i32);
                    }
                });
            }
            _ => todo!(),
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
