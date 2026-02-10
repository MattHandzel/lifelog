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

    pub async fn process_replay(
        &self,
        req: lifelog_types::ReplayRequest,
    ) -> Result<lifelog_types::ReplayResponse, LifelogError> {
        let server = self.server.read().await;
        server.process_replay(req).await
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

    pub async fn handle_clock_sample(
        &self,
        collector_id: &str,
        device_now: chrono::DateTime<Utc>,
        backend_now: Option<chrono::DateTime<Utc>>,
    ) {
        let server = self.server.read().await;
        let backend_now = backend_now.unwrap_or_else(Utc::now);

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
        // NOTE: `surrealdb::engine::remote::ws::Ws` expects an address like `127.0.0.1:8000`
        // (tests use this form). Prefixing with `ws://` can hang depending on driver/version.
        let db = Surreal::new::<Ws>(&config.database_endpoint)
            .await
            .map_err(|e| LifelogError::Database(e.to_string()))?;

        let db_user = std::env::var("LIFELOG_DB_USER").map_err(|_| LifelogError::Validation {
            field: "LIFELOG_DB_USER".to_string(),
            reason: "must be set (SurrealDB root username)".to_string(),
        })?;
        let db_pass = std::env::var("LIFELOG_DB_PASS").map_err(|_| LifelogError::Validation {
            field: "LIFELOG_DB_PASS".to_string(),
            reason: "must be set (SurrealDB root password)".to_string(),
        })?;

        db.signin(Root {
            username: &db_user,
            password: &db_pass,
        })
        .await
        .map_err(|e| LifelogError::Database(e.to_string()))?;

        db.use_ns("lifelog")
            .use_db(&config.database_name)
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
        let scoped_origins: Vec<DataOrigin> = if query_msg.search_origins.is_empty() {
            available_origins.clone()
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

        // LLQL (JSON) escape hatch: allow the UI to execute typed cross-modal queries
        // without changing the protobuf. Use `Query.text = ["llql:{...json...}"]`.
        if let Some(ast_query) = crate::query::llql::try_parse_llql(&query_msg.text)? {
            let plan = crate::query::planner::Planner::plan(&ast_query, &scoped_origins);
            return crate::query::executor::execute(&self.db, plan)
                .await
                .map_err(|e| LifelogError::Database(format!("query execution failed: {e}")));
        }

        for origin in scoped_origins {
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

            // Pass the full catalog so temporal operators like WITHIN can resolve other streams.
            let plan = crate::query::planner::Planner::plan(&query, &available_origins);
            match crate::query::executor::execute(&self.db, plan).await {
                Ok(res) => keys.extend(res),
                Err(e) => tracing::error!("Query execution failed for {}: {}", table, e),
            }
        }

        Ok(keys)
    }

    async fn process_replay(
        &self,
        req: lifelog_types::ReplayRequest,
    ) -> Result<lifelog_types::ReplayResponse, LifelogError> {
        let window = req.window.ok_or(LifelogError::Validation {
            field: "window".to_string(),
            reason: "must be provided".to_string(),
        })?;

        let start_pb = window.start.ok_or(LifelogError::Validation {
            field: "window.start".to_string(),
            reason: "must be provided".to_string(),
        })?;
        let end_pb = window.end.ok_or(LifelogError::Validation {
            field: "window.end".to_string(),
            reason: "must be provided".to_string(),
        })?;

        let start = chrono::DateTime::from_timestamp(start_pb.seconds, start_pb.nanos as u32)
            .unwrap_or(chrono::DateTime::<Utc>::MIN_UTC);
        let end = chrono::DateTime::from_timestamp(end_pb.seconds, end_pb.nanos as u32)
            .unwrap_or(chrono::DateTime::<Utc>::MAX_UTC);

        if start >= end {
            return Err(LifelogError::Validation {
                field: "window".to_string(),
                reason: "start must be < end".to_string(),
            });
        }

        let max_steps = if req.max_steps == 0 {
            500usize
        } else {
            req.max_steps as usize
        };
        let max_context_per_step = if req.max_context_per_step == 0 {
            50usize
        } else {
            req.max_context_per_step as usize
        };

        let pad_ms = req.context_pad_ms.min(5 * 60 * 1000); // hard cap at 5 minutes
        let pad = chrono::Duration::milliseconds(pad_ms as i64);

        let available_origins = get_origins_from_db(&self.db).await?;

        let screen_origin = if req.screen_origin.trim().is_empty() {
            available_origins
                .iter()
                .find(|o| o.modality_name == "Screen")
                .cloned()
                .ok_or_else(|| LifelogError::Validation {
                    field: "screen_origin".to_string(),
                    reason: "no Screen origins available".to_string(),
                })?
        } else {
            available_origins
                .iter()
                .find(|o| {
                    o.get_table_name() == req.screen_origin
                        || o.to_string() == req.screen_origin
                        || o.modality_name == req.screen_origin
                })
                .cloned()
                .or_else(|| DataOrigin::tryfrom_string(req.screen_origin.clone()).ok())
                .ok_or_else(|| LifelogError::Validation {
                    field: "screen_origin".to_string(),
                    reason: format!("unknown origin '{}'", req.screen_origin),
                })?
        };

        // Phase 1: load screen frames in the window.
        #[derive(serde::Deserialize, Debug)]
        struct ScreenRow {
            uuid: String,
            t_canonical: Option<surrealdb::sql::Datetime>,
        }

        let screen_table = screen_origin.get_table_name();
        let sql = format!(
            "SELECT uuid, t_canonical FROM `{}` WHERE t_canonical >= d'{}' AND t_canonical < d'{}' ORDER BY t_canonical ASC LIMIT {};",
            screen_table,
            start.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            end.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            max_steps
        );
        let mut resp = self
            .db
            .query(sql)
            .await
            .map_err(|e| LifelogError::Database(format!("replay screen query failed: {e}")))?;
        let rows: Vec<ScreenRow> = resp
            .take(0)
            .map_err(|e| LifelogError::Database(format!("replay screen take failed: {e}")))?;

        let screen_frames: Vec<(String, chrono::DateTime<Utc>)> = rows
            .into_iter()
            .filter_map(|r| Some((r.uuid, r.t_canonical?.0)))
            .collect();

        let mut steps =
            crate::replay::build_replay_steps_for_screen(screen_frames, screen_origin.clone(), end);

        if steps.is_empty() || req.context_origins.is_empty() {
            let proto_steps = steps
                .into_iter()
                .map(|s| lifelog_types::ReplayStep {
                    start: lifelog_types::to_pb_ts(s.start),
                    end: lifelog_types::to_pb_ts(s.end),
                    screen_key: Some(lifelog_types::LifelogDataKey::from(s.screen_key)),
                    context_keys: Vec::new(),
                })
                .collect();
            return Ok(lifelog_types::ReplayResponse { steps: proto_steps });
        }

        // Phase 2: load context records once per origin (bounded), then assign to steps.
        #[derive(serde::Deserialize, Debug)]
        struct CtxRow {
            uuid: String,
            t_canonical: Option<surrealdb::sql::Datetime>,
            t_end: Option<surrealdb::sql::Datetime>,
        }

        let mut ctx_records: Vec<crate::replay::IntervalKey> = Vec::new();

        // Per-origin bound: cap by (steps * per_step), but clamp to keep the query bounded.
        let per_origin_limit = (max_steps * max_context_per_step).min(10_000);

        let window_start = start - pad;
        let window_end = end + pad;

        for s in &req.context_origins {
            let resolved: Vec<DataOrigin> = if s.trim().is_empty() {
                Vec::new()
            } else if s == "*" {
                available_origins.clone()
            } else {
                let mut out = Vec::new();
                for o in &available_origins {
                    if o.get_table_name() == *s || o.to_string() == *s || o.modality_name == *s {
                        out.push(o.clone());
                    }
                }
                if out.is_empty() {
                    if let Ok(o) = DataOrigin::tryfrom_string(s.clone()) {
                        out.push(o);
                    }
                }
                out
            };

            for origin in resolved {
                let table = origin.get_table_name();
                let sql = format!(
                    "SELECT uuid, t_canonical, t_end FROM `{}` WHERE t_canonical <= d'{}' AND t_end >= d'{}' ORDER BY t_canonical ASC LIMIT {};",
                    table,
                    window_end.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                    window_start.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                    per_origin_limit
                );
                let mut resp = self.db.query(sql).await.map_err(|e| {
                    LifelogError::Database(format!("replay context query failed for {table}: {e}"))
                })?;
                let rows: Vec<CtxRow> = resp.take(0).map_err(|e| {
                    LifelogError::Database(format!("replay context take failed for {table}: {e}"))
                })?;

                for r in rows {
                    let Some(t0) = r.t_canonical else { continue };
                    let start = t0.0;
                    let end = r.t_end.map(|dt| dt.0).unwrap_or(start);
                    if let Ok(uuid) = r.uuid.parse() {
                        ctx_records.push(crate::replay::IntervalKey {
                            key: LifelogFrameKey {
                                uuid,
                                origin: origin.clone(),
                            },
                            start,
                            end,
                        });
                    }
                }
            }
        }

        crate::replay::assign_context_keys(&mut steps, ctx_records, pad, max_context_per_step);

        let proto_steps = steps
            .into_iter()
            .map(|s| lifelog_types::ReplayStep {
                start: lifelog_types::to_pb_ts(s.start),
                end: lifelog_types::to_pb_ts(s.end),
                screen_key: Some(lifelog_types::LifelogDataKey::from(s.screen_key)),
                context_keys: s
                    .context_keys
                    .into_iter()
                    .map(lifelog_types::LifelogDataKey::from)
                    .collect(),
            })
            .collect();

        Ok(lifelog_types::ReplayResponse { steps: proto_steps })
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
