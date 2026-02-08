use crate::policy::*;
use chrono::Utc;
use config::ServerPolicyConfig;
use config::{ServerConfig, SystemConfig};
use lifelog_core::*;
use lifelog_types::DataModality;
use lifelog_types::*;
use lifelog_types::{CollectorState, SystemState};
use std::collections::HashMap;
use std::sync::Arc;
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

/// (device_time, server_time) pairs for clock skew estimation.
type SkewSamples = HashMap<String, Vec<(chrono::DateTime<Utc>, chrono::DateTime<Utc>)>>;

#[derive(Debug, Clone)]
pub struct Server {
    pub(crate) db: Surreal<Client>,
    pub(crate) cas: FsCas,
    pub(crate) config: Arc<ServerConfig>,
    pub(crate) state: Arc<RwLock<SystemState>>,
    pub(crate) registered_collectors: Arc<RwLock<Vec<RegisteredCollector>>>,
    pub(crate) policy: Arc<RwLock<ServerPolicy>>,
    transforms: Arc<RwLock<Vec<LifelogTransform>>>,
    pub(crate) skew_estimates: Arc<RwLock<HashMap<String, lifelog_core::time_skew::SkewEstimate>>>,
    pub(crate) skew_samples: Arc<RwLock<SkewSamples>>,
}

#[derive(Clone)]
pub struct ServerHandle {
    pub server: Arc<RwLock<Server>>,
}

impl ServerHandle {
    pub fn new(server: Arc<RwLock<Server>>) -> Self {
        ServerHandle { server }
    }

    pub async fn r#loop(&self) {
        loop {
            {
                let server = self.server.read().await;
                server.step().await;
            }
            tokio::time::sleep(time::Duration::from_millis(100)).await;
        }
    }

    pub async fn contains_collector(&self, collector_name: String) -> bool {
        let server = self.server.read().await;
        server.contains_collector(collector_name).await
    }

    pub async fn get_state(&self) -> SystemState {
        let server = self.server.read().await;
        server.get_state().await
    }

    pub async fn get_config(&self) -> SystemConfig {
        let server = self.server.read().await;
        server.get_config().await
    }

    pub async fn get_data(
        &self,
        keys: Vec<lifelog_types::LifelogDataKey>,
    ) -> Result<Vec<lifelog_types::LifelogData>, LifelogError> {
        let server = self.server.read().await;
        let mut data = Vec::new();
        for key in keys {
            let core_key = LifelogFrameKey::new(
                key.uuid.parse().unwrap_or_default(),
                DataOrigin::tryfrom_string(key.origin).unwrap_or_else(|_| {
                    DataOrigin::new(
                        DataOriginType::DeviceId("unknown".to_string()),
                        "unknown".to_string(),
                    )
                }),
            );
            if let Ok(d) = get_data_by_key(&server.db, &server.cas, &core_key).await {
                data.push(d);
            }
        }
        Ok(data)
    }

    pub async fn process_query(
        &self,
        query: lifelog_types::Query,
    ) -> Result<Vec<LifelogFrameKey>, LifelogError> {
        let server = self.server.read().await;
        server.process_query(query).await
    }

    pub async fn register_collector(&self, collector: RegisteredCollector) {
        let server = self.server.write().await;
        server.registered_collectors.write().await.push(collector);
    }

    pub async fn report_collector_state(&self, state: CollectorState) {
        let server = self.server.write().await;
        server
            .state
            .write()
            .await
            .collector_states
            .insert(state.name.clone(), state);
    }

    pub async fn remove_collector(&self, id: &str) {
        let server = self.server.write().await;
        server
            .registered_collectors
            .write()
            .await
            .retain(|c| c.id != id);
    }

    pub async fn handle_clock_sample(&self, collector_id: &str, device_now: chrono::DateTime<Utc>) {
        let server = self.server.read().await;
        let backend_now = Utc::now();

        const MAX_SKEW_SAMPLES: usize = 20;

        let estimate = {
            let mut samples = server.skew_samples.write().await;
            let entry = samples.entry(collector_id.to_string()).or_default();
            entry.push((device_now, backend_now));
            if entry.len() > MAX_SKEW_SAMPLES {
                entry.drain(..entry.len() - MAX_SKEW_SAMPLES);
            }
            lifelog_core::time_skew::estimate_skew(entry)
        }; // skew_samples write guard dropped before acquiring skew_estimates

        server
            .skew_estimates
            .write()
            .await
            .insert(collector_id.to_string(), estimate);
    }
}

impl Server {
    pub async fn new(config: &ServerConfig) -> Result<Self, LifelogError> {
        let db_endpoint = format!("ws://{}", config.database_endpoint);
        let db = Surreal::new::<Ws>(db_endpoint)
            .await
            .map_err(|e| LifelogError::Database(e.to_string()))?;

        db.signin(Root {
            username: "root",
            password: "root",
        })
        .await
        .map_err(|e| LifelogError::Database(e.to_string()))?;

        db.use_ns("lifelog")
            .use_db("test_db")
            .await
            .map_err(|e| LifelogError::Database(e.to_string()))?;

        crate::schema::run_startup_migrations(&db).await?;

        let cas = FsCas::new(&config.cas_path);

        let system_state = SystemState {
            collector_states: HashMap::new(),
            interface_states: HashMap::new(),
            server_state: Some(ServerState {
                name: "Lifelog Server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                timestamp: None,
                uptime_since: None,
                memory_usage: 0.0,
                cpu_usage: 0.0,
                threads: 0.0,
                total_frames_stored: 0,
                disk_usage_bytes: 0,
                pending_actions: vec![],
                timestamp_of_last_sync: None,
            }),
        };

        let policy_config = ServerPolicyConfig {
            collector_sync_interval: 10.0, // Default 10s
            max_cpu_usage: UsageType::Percentage(80.0),
            max_memory_usage: UsageType::RealValue(1024, lifelog_core::Unit::GB),
            max_threads: UsageType::RealValue(10, lifelog_core::Unit::Count),
        };

        let ocr_config = data_modalities::ocr::OcrConfig {
            language: "eng".to_string(),
            engine_path: None,
        };

        let ocr_transform = data_modalities::ocr::OcrTransform::new(
            DataOrigin::new(
                DataOriginType::DeviceId("FF:FF:FF:FF:FF:FF".to_string()),
                DataModality::Screen.as_str_name().to_string(),
            ),
            ocr_config,
        );

        Ok(Server {
            db,
            cas,
            config: Arc::new(config.clone()),
            state: Arc::new(RwLock::new(system_state)),
            registered_collectors: Arc::new(RwLock::new(vec![])),
            policy: Arc::new(RwLock::new(ServerPolicy::new(policy_config))),
            transforms: Arc::new(RwLock::new(vec![ocr_transform.into()])),
            skew_estimates: Arc::new(RwLock::new(HashMap::new())),
            skew_samples: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn step(&self) {
        let state = self.get_state().await;

        let action = {
            let policy = self.policy.read().await;
            policy.get_action(state.clone())
        };

        self.add_audit_log(&action).await;
        self.do_action(action, state).await;
    }

    async fn contains_collector(&self, collector_name: String) -> bool {
        let collectors = self.registered_collectors.read().await;
        for collector in collectors.iter() {
            if collector.id == collector_name {
                return true;
            }
        }
        false
    }

    async fn get_state(&self) -> SystemState {
        self.state.read().await.clone()
    }

    async fn get_config(&self) -> SystemConfig {
        let collectors = self.registered_collectors.read().await;
        let mut collector_configs = HashMap::new();
        for collector in collectors.iter() {
            if let Some(config) = &collector.latest_config {
                collector_configs.insert(collector.id.clone(), config.clone());
            }
        }

        SystemConfig {
            server: Some((*self.config).clone()),
            collectors: collector_configs,
        }
    }

    pub async fn get_skew_estimate(
        &self,
        collector_id: &str,
    ) -> Option<lifelog_core::time_skew::SkewEstimate> {
        self.skew_estimates.read().await.get(collector_id).copied()
    }

    async fn process_query(
        &self,
        query_msg: lifelog_types::Query,
    ) -> Result<Vec<LifelogFrameKey>, LifelogError> {
        let mut keys: Vec<LifelogFrameKey> = vec![];

        let available_origins = get_origins_from_db(&self.db).await?;
        let target_origins: Vec<DataOrigin> = if query_msg.search_origins.is_empty() {
            available_origins
        } else {
            let mut resolved = Vec::new();
            for s in &query_msg.search_origins {
                if let Some(o) = available_origins
                    .iter()
                    .find(|o| o.get_table_name() == *s || o.to_string() == *s)
                {
                    resolved.push(o.clone());
                } else if let Ok(o) = DataOrigin::tryfrom_string(s.clone()) {
                    resolved.push(o);
                } else {
                    // Try to match as modality name
                    for o in &available_origins {
                        if o.modality_name == *s {
                            resolved.push(o.clone());
                        }
                    }
                }
            }
            resolved
        };

        for origin in target_origins {
            let table = origin.get_table_name();
            let modality = origin.modality_name.clone();

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

            let mut text_expr = None;
            for text in &query_msg.text {
                if text == "*" {
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

            let query = crate::query::ast::Query {
                target: crate::query::ast::StreamSelector::StreamId(table.clone()),
                filter,
            };

            let plan = crate::query::planner::Planner::plan(&query, &[origin.clone()]);
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

    async fn do_action(&self, action: ServerAction, state: SystemState) {
        match action {
            ServerAction::Sleep(duration) => {
                tokio::time::sleep(duration).await;
            }
            ServerAction::SyncData(query) => {
                {
                    if let Some(ss) = self.state.write().await.server_state.as_mut() {
                        ss.pending_actions.push(ServerActionType::SyncData as i32);
                    }
                }
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
                let cas_clone = self.cas.clone();
                let transforms = self.transforms.clone().read().await.to_vec();
                tokio::spawn(async move {
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
                            50,
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
                            &cas_clone,
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
                        }
                    }

                    let mut state = state_clone.write().await;
                    if let Some(ss) = state.server_state.as_mut() {
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
