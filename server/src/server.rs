use crate::policy::*;
use chrono::Utc;
use config::ServerPolicyConfig;
use config::{CollectorConfig, ServerConfig, SystemConfig};
use data_modalities::*;
use lifelog_core::*;
use lifelog_types::DataModality;
use lifelog_types::*;
use lifelog_types::{CollectorState, SystemState};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::time;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::RwLock;
use utils::cas::FsCas;

use crate::data_retrieval::get_data_by_key;
use crate::db::get_origins_from_db;
use crate::sync::sync_data_with_collectors;
use crate::transform::LifelogTransform;

pub type ServerAction = lifelog_core::ServerAction<
    lifelog_types::QueryRequest,
    lifelog_types::GetDataRequest,
    lifelog_types::Uuid,
>;

pub type RegisteredCollector =
    lifelog_core::RegisteredCollector<lifelog_types::ServerCommand, lifelog_types::CollectorConfig>;

// Re-export for external consumers (main.rs, tests)
pub use crate::grpc_service::GRPCServerLifelogServerService;

#[derive(Clone)]
pub struct ServerHandle {
    pub server: Arc<RwLock<Server>>,
}

impl ServerHandle {
    pub fn new(server: Arc<RwLock<Server>>) -> Self {
        Self { server }
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

    pub async fn remove_collector(&self, collector_id: &str) {
        let server_lock = self.server.read().await;
        let mut collectors_vec = server_lock.registered_collectors.write().await;
        collectors_vec.retain(|c| c.id != collector_id);
        tracing::info!(id = %collector_id, "Collector removed");
    }

    pub async fn contains_collector(&self, collector_name: String) -> bool {
        let server = self.server.read().await;
        server.contains_collector(collector_name).await
    }

    pub async fn report_collector_state(&self, state: CollectorState) -> Result<(), LifelogError> {
        let server = self.server.write().await;
        server.report_collector_state(state).await
    }

    pub async fn process_query(
        &self,
        query: lifelog_types::Query,
    ) -> Result<Vec<LifelogFrameKey>, LifelogError> {
        tracing::debug!(?query, "Requesting server lock for process_query");
        self.server.read().await.process_query(query).await
    }

    pub async fn get_system_config(&self) -> Result<SystemConfig, LifelogError> {
        let server = self.server.read().await;
        server.get_system_config().await
    }

    pub async fn get_data(
        &self,
        keys: Vec<LifelogFrameKey>,
    ) -> Result<Vec<lifelog_types::LifelogData>, LifelogError> {
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
    #[allow(dead_code)]
    origins: Arc<RwLock<Vec<DataOrigin>>>,
    transforms: Arc<RwLock<Vec<LifelogTransform>>>, // TODO: These should be registered transforms
    config: ServerConfig,
    pub(crate) cas: FsCas,
    started_at: chrono::DateTime<Utc>,
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
                version: env!("CARGO_PKG_VERSION").to_string(),
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
                DataModality::Screen.as_str_name().to_string(),
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
            started_at: Utc::now(),
        };

        Ok(s)
    }
    async fn get_system_config(&self) -> Result<SystemConfig, LifelogError> {
        // Get the config of all the collectors from the cached state
        let collectors = self.registered_collectors.read().await;
        let mut collector_configs: HashMap<String, CollectorConfig> = HashMap::new();
        for collector in collectors.iter() {
            if let Some(config) = &collector.latest_config {
                collector_configs.insert(collector.id.clone(), config.clone());
            }
        }
        let config = SystemConfig {
            server: Some(self.config.clone()),
            collectors: collector_configs,
        };

        Ok(config)
    }
    async fn report_collector_state(&self, mut state: CollectorState) -> Result<(), LifelogError> {
        state.last_seen = Some(Utc::now().into());
        let mut system_state = self.state.write().await;
        system_state
            .collector_states
            .insert(state.name.clone(), state);
        Ok(())
    }

    async fn get_data(
        &self,
        req: Vec<LifelogFrameKey>,
    ) -> Result<Vec<lifelog_types::LifelogData>, LifelogError> {
        let mut datas: Vec<lifelog_types::LifelogData> = vec![];
        for key in req.iter() {
            let data: lifelog_types::LifelogData =
                get_data_by_key(&self.db, key).await.map_err(|e| {
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

        if (t_now - t_last).num_seconds() as f64 >= (self.config.collector_sync_interval as f64)
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
        }
    }
}

// TODO: refactor the server to be an actor model instead of RwLock
// https://softwarepatternslexicon.com/patterns-rust/9/14/
impl Server {
    /// This function will be run upon startup, it will handle the server's main loop of doing
    /// actions
    ///
    // TODO: Refactor this so do_action (a blocking task) isn't running here, we don't wanna hold
    // onto the lock
    pub async fn step(&self) {
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
        self.policy.clone()
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

    #[allow(clippy::expect_used)]
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
            ss.uptime_since = Some(self.started_at.into());
        }
        state.clone()
    }

    async fn process_query(
        &self,
        query_msg: lifelog_types::Query,
    ) -> Result<Vec<LifelogFrameKey>, LifelogError> {
        let mut keys: Vec<LifelogFrameKey> = vec![];

        // Determine target origins
        let target_origins: Vec<DataOrigin> = if query_msg.search_origins.is_empty() {
            get_origins_from_db(&self.db).await?
        } else {
            query_msg
                .search_origins
                .into_iter()
                .filter_map(|s| DataOrigin::tryfrom_string(s).ok())
                .collect()
        };

        for origin in target_origins {
            let table = origin.get_table_name();
            let modality = origin.modality_name.clone();

            // Build filter
            // 1. Time ranges (OR)
            let mut time_expr = None;
            for tr in &query_msg.time_ranges {
                let start = tr
                    .start
                    .as_ref()
                    .map(|t| {
                        chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
                            .unwrap_or_default()
                    })
                    .unwrap_or(chrono::DateTime::<Utc>::MIN_UTC);
                let end = tr
                    .end
                    .as_ref()
                    .map(|t| {
                        chrono::DateTime::from_timestamp(t.seconds, t.nanos as u32)
                            .unwrap_or_default()
                    })
                    .unwrap_or(chrono::DateTime::<Utc>::MAX_UTC);
                let range = crate::query::ast::Expression::TimeRange(start, end);
                time_expr = match time_expr {
                    Some(e) => Some(crate::query::ast::Expression::Or(
                        Box::new(e),
                        Box::new(range),
                    )),
                    None => Some(range),
                };
            }

            // 2. Text (OR)
            let mut text_expr = None;
            for text in &query_msg.text {
                if text == "*" {
                    // Wildcard: match everything (effectively no text filter)
                    continue;
                }
                let field = match modality.as_str() {
                    "Browser" => "title",
                    "ShellHistory" => "command",
                    "WindowActivity" => "window_title",
                    _ => "text",
                };
                let contains =
                    crate::query::ast::Expression::Contains(field.to_string(), text.clone());
                text_expr = match text_expr {
                    Some(e) => Some(crate::query::ast::Expression::Or(
                        Box::new(e),
                        Box::new(contains),
                    )),
                    None => Some(contains),
                };
            }

            // Combine Time AND Text
            let filter = match (time_expr, text_expr) {
                (Some(t), Some(txt)) => {
                    crate::query::ast::Expression::And(Box::new(t), Box::new(txt))
                }
                (Some(t), None) => t,
                (None, Some(txt)) => txt,
                (None, None) => crate::query::ast::Expression::TimeRange(
                    chrono::DateTime::<Utc>::MIN_UTC,
                    chrono::DateTime::<Utc>::MAX_UTC,
                ),
            };

            // Directly build the SQL for this specific table
            let where_clause = crate::query::planner::Planner::compile_expression(&filter);
            let sql = format!("SELECT * FROM `{}` WHERE {};", table, where_clause);

            let plan = crate::query::planner::ExecutionPlan::SimpleQuery(sql);
            match crate::query::executor::execute(&self.db, plan).await {
                Ok(res) => keys.extend(res),
                Err(e) => tracing::error!("Query execution failed for {}: {}", table, e),
            }
        }

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
                    for transform in transforms {
                        let id = transform.id();
                        let watermark = match crate::db::get_watermark(&db_connection, &id).await {
                            Ok(w) => w,
                            Err(e) => {
                                tracing::error!("Failed to get watermark for {}: {}", id, e);
                                continue;
                            }
                        };

                        let keys = match crate::data_retrieval::get_keys_after_timestamp(
                            &db_connection,
                            &transform.source(),
                            watermark,
                            50, // Bounded batch
                        )
                        .await
                        {
                            Ok(k) => k,
                            Err(e) => {
                                tracing::error!("Failed to get keys for {}: {}", id, e);
                                continue;
                            }
                        };

                        if keys.is_empty() {
                            continue;
                        }

                        if let Some(last_ts) = crate::transform::transform_data_single(
                            &db_connection,
                            &keys,
                            &transform,
                        )
                        .await
                        {
                            if let Err(e) =
                                crate::db::set_watermark(&db_connection, &id, last_ts).await
                            {
                                tracing::error!("Failed to set watermark for {}: {}", id, e);
                            }
                        } else {
                        }
                    }

                    if let Some(ss) = state_clone.write().await.server_state.as_mut() {
                        ss.pending_actions
                            .retain(|&a| a != ServerActionType::TransformData as i32);
                    }
                });
            }
            #[allow(clippy::todo)]
            _ => todo!(),
        }
    }
}
