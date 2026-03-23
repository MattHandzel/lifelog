use crate::policy::*;
use crate::postgres::{connect_pool, run_migrations, PostgresPool};
use chrono::Utc;
use config::ServerPolicyConfig;
use config::{ServerConfig, SystemConfig};
use lifelog_core::*;
use lifelog_types::*;
use lifelog_types::{CollectorState, SystemState};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time;
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::RwLock;
use utils::cas::FsCas;

use crate::data_retrieval::get_data_by_key;
use crate::db::get_origins_from_db;
use crate::retention::prune_once;
use crate::sync::sync_data_with_collectors;
use crate::transform::{dag::TransformDag, TransformExecutor};

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
    pub(crate) postgres_pool: PostgresPool,
    pub(crate) cas: FsCas,
    pub(crate) config: Arc<RwLock<ServerConfig>>,
    pub(crate) state: Arc<RwLock<SystemState>>,
    pub(crate) registered_collectors: Arc<RwLock<Vec<RegisteredCollector>>>,
    pub(crate) policy: Arc<RwLock<ServerPolicy>>,
    pub(crate) transform_dag: Arc<TransformDag>,
    pub(crate) http_client: reqwest::Client,
    pub(crate) skew_estimates: Arc<RwLock<HashMap<String, lifelog_core::time_skew::SkewEstimate>>>,
    pub(crate) skew_samples: Arc<RwLock<SkewSamples>>,
}

#[derive(Clone)]
pub struct ServerHandle {
    pub server: Arc<RwLock<Server>>,
}

fn normalize_collector_id(id: &str) -> String {
    id.replace(":", "")
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

    pub async fn apply_system_config(
        &self,
        system_config: SystemConfig,
    ) -> Result<(), LifelogError> {
        let server = self.server.read().await;
        server.apply_system_config(system_config).await
    }

    pub async fn run_retention_once(
        &self,
    ) -> Result<crate::retention::RetentionRunSummary, LifelogError> {
        let server = self.server.read().await;
        server.run_retention_once().await
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
                DataOrigin::tryfrom_string(key.origin.clone()).unwrap_or_else(|_| {
                    DataOrigin::new(
                        DataOriginType::DeviceId("unknown".to_string()),
                        "unknown".to_string(),
                    )
                }),
            );
            match get_data_by_key(
                &server.db,
                Some(&server.postgres_pool),
                &server.cas,
                &core_key,
            )
            .await
            {
                Ok(d) => data.push(d),
                Err(e) => {
                    tracing::error!(
                        uuid = %key.uuid,
                        origin = %key.origin,
                        error = %e,
                        "Failed to get data by key"
                    );
                }
            }
        }
        Ok(data)
    }

    pub async fn list_postgres_origins(&self) -> Vec<DataOrigin> {
        let server = self.server.read().await;
        let pool = &server.postgres_pool;

        let client = match pool.get().await {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        let mut origins = Vec::new();
        // Check screen_records
        if let Ok(rows) = client
            .query("SELECT DISTINCT collector_id FROM screen_records", &[])
            .await
        {
            for row in rows {
                let id: String = row.get(0);
                origins.push(DataOrigin {
                    origin: DataOriginType::DeviceId(id),
                    modality_name: "Screen".to_string(),
                });
            }
        }
        // Check all unique collectors in upload_chunks
        if let Ok(rows) = client
            .query("SELECT DISTINCT collector_id FROM upload_chunks", &[])
            .await
        {
            for row in rows {
                let id: String = row.get(0);
                // Also report as Processes if we see it
                origins.push(DataOrigin {
                    origin: DataOriginType::DeviceId(id),
                    modality_name: "Processes".to_string(),
                });
            }
        }
        origins
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
    async fn resolve_identity_candidates(&self, identifier: &str) -> Vec<String> {
        let collectors = self.registered_collectors.read().await;
        let mut out = Vec::new();

        for collector in collectors.iter() {
            let alias = collector
                .latest_config
                .as_ref()
                .map(|c| c.id.trim())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let mac = collector.id.clone();

            let alias_match = alias
                .as_deref()
                .map(|a| a.eq_ignore_ascii_case(identifier))
                .unwrap_or(false);
            let mac_match = mac.eq_ignore_ascii_case(identifier);

            if alias_match || mac_match {
                // Alias should take precedence for query resolution when both exist.
                if let Some(a) = alias {
                    out.push(a);
                }
                out.push(mac);
            }
        }

        let mut seen = HashSet::new();
        out.retain(|v| seen.insert(v.to_lowercase()));
        out
    }

    async fn resolve_search_origins(
        &self,
        search_term: &str,
        available_origins: &[DataOrigin],
    ) -> Vec<DataOrigin> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();

        let push_unique =
            |acc: &mut Vec<DataOrigin>, seen: &mut HashSet<String>, o: &DataOrigin| {
                let key = o.get_table_name();
                if seen.insert(key) {
                    acc.push(o.clone());
                }
            };

        // 1) Direct table/origin/modality matches.
        for o in available_origins {
            if o.get_table_name() == search_term
                || o.to_string() == search_term
                || o.modality_name == search_term
            {
                push_unique(&mut out, &mut seen, o);
            }
        }

        // 2) Identity-aware mapping (alias or MAC), optionally with modality suffix.
        if let Some((device_part, modality_part)) = search_term.split_once(':') {
            let candidates = self.resolve_identity_candidates(device_part.trim()).await;
            for candidate in candidates {
                let wanted = format!("{}:{}", candidate, modality_part.trim());
                for o in available_origins {
                    if o.get_table_name().eq_ignore_ascii_case(&wanted)
                        || o.to_string().eq_ignore_ascii_case(&wanted)
                    {
                        push_unique(&mut out, &mut seen, o);
                    }
                }
            }
        } else {
            let candidates = self.resolve_identity_candidates(search_term.trim()).await;
            if !candidates.is_empty() {
                for o in available_origins {
                    let table = o.get_table_name();
                    let device = table.split(':').next().unwrap_or_default();
                    if candidates.iter().any(|c| c.eq_ignore_ascii_case(device)) {
                        push_unique(&mut out, &mut seen, o);
                    }
                }
            }
        }

        // 3) Fallback parser support for explicit DataOrigin strings.
        if out.is_empty() {
            if let Ok(parsed) = DataOrigin::tryfrom_string(search_term.to_string()) {
                out.push(parsed);
            }
        }

        out
    }

    fn postgres_table_for_modality(modality: &str) -> Option<&'static str> {
        match modality {
            "Screen" => Some("screen_records"),
            "Browser" => Some("browser_records"),
            "Ocr" => Some("ocr_records"),
            "Audio" => Some("audio_records"),
            "Clipboard" => Some("clipboard_records"),
            "ShellHistory" => Some("shell_history_records"),
            "Keystrokes" => Some("keystroke_records"),
            _ => None,
        }
    }

    fn collector_id_from_origin(origin: &DataOrigin) -> Option<&str> {
        match &origin.origin {
            DataOriginType::DeviceId(id) => Some(id.as_str()),
            DataOriginType::DataOrigin(parent) => Self::collector_id_from_origin(parent),
        }
    }

    async fn get_origins_from_postgres(
        pool: &PostgresPool,
    ) -> Result<Vec<DataOrigin>, LifelogError> {
        let client = pool
            .get()
            .await
            .map_err(|e| LifelogError::Database(format!("postgres pool get failed: {e}")))?;

        let modality_tables = [
            ("Screen", "screen_records"),
            ("Browser", "browser_records"),
            ("Ocr", "ocr_records"),
            ("Audio", "audio_records"),
            ("Clipboard", "clipboard_records"),
            ("ShellHistory", "shell_history_records"),
            ("Keystrokes", "keystroke_records"),
        ];

        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for (modality, table) in modality_tables {
            let sql = format!("SELECT DISTINCT collector_id FROM {table}");
            let rows = client.query(sql.as_str(), &[]).await.map_err(|e| {
                LifelogError::Database(format!("postgres origin query failed for {table}: {e}"))
            })?;
            for row in rows {
                let collector_id: String = row.get("collector_id");
                let origin =
                    DataOrigin::new(DataOriginType::DeviceId(collector_id), modality.into());
                let key = origin.get_table_name();
                if seen.insert(key) {
                    out.push(origin);
                }
            }
        }
        Ok(out)
    }

    async fn get_available_origins_hybrid(&self) -> Result<Vec<DataOrigin>, LifelogError> {
        let mut origins = get_origins_from_db(&self.db).await?;
        {
            let pg_origins = Self::get_origins_from_postgres(&self.postgres_pool).await?;
            let mut seen: HashSet<String> = origins.iter().map(|o| o.get_table_name()).collect();
            for origin in pg_origins {
                let key = origin.get_table_name();
                if seen.insert(key) {
                    origins.push(origin);
                }
            }
        }
        Ok(origins)
    }

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
        let postgres_url = std::env::var("LIFELOG_POSTGRES_INGEST_URL").map_err(|_| {
            LifelogError::Database(
                "LIFELOG_POSTGRES_INGEST_URL must be set — PostgreSQL is required".to_string(),
            )
        })?;
        if postgres_url.trim().is_empty() {
            return Err(LifelogError::Database(
                "LIFELOG_POSTGRES_INGEST_URL must not be empty — PostgreSQL is required"
                    .to_string(),
            ));
        }
        let max_connections = std::env::var("LIFELOG_POSTGRES_INGEST_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(16);
        let postgres_pool = connect_pool(&postgres_url, max_connections).await?;
        run_migrations(&postgres_pool).await?;
        tracing::info!(
            max_connections,
            "Postgres backend enabled via LIFELOG_POSTGRES_INGEST_URL"
        );

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
                postgres_pool_enabled: false,
                postgres_pool_max_size: 0,
                postgres_pool_size: 0,
                postgres_pool_available: 0,
                postgres_pool_waiting: 0,
            }),
        };

        let policy_config = ServerPolicyConfig {
            collector_sync_interval: 10.0, // Default 10s
            max_cpu_usage: UsageType::Percentage(80.0),
            max_memory_usage: UsageType::RealValue(1024, lifelog_core::Unit::GB),
            max_threads: UsageType::RealValue(10, lifelog_core::Unit::Count),
        };

        let mut executors: Vec<Arc<dyn TransformExecutor>> = Vec::new();

        for spec in &config.transforms {
            if !spec.enabled {
                continue;
            }

            let transform_type = if spec.transform_type.is_empty() {
                spec.id.as_str()
            } else {
                spec.transform_type.as_str()
            };

            let source = match DataOrigin::tryfrom_string(spec.source_origin.clone()) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(
                        source_origin = %spec.source_origin,
                        error = %e,
                        "Skipping transform with invalid source_origin"
                    );
                    continue;
                }
            };

            match transform_type {
                "ocr" => {
                    let ocr_config = data_modalities::ocr::OcrConfig {
                        language: spec.language.clone().unwrap_or_else(|| "eng".to_string()),
                        engine_path: None,
                    };
                    let executor = crate::transform::ocr::OcrExecutor::new(source, ocr_config)
                        .with_id(spec.id.clone());
                    executors.push(Arc::new(executor));
                }
                "stt" => {
                    let executor = crate::transform::stt::SttExecutor::new(
                        spec.id.clone(),
                        source,
                        spec.service_endpoint.clone(),
                        &spec.params,
                    );
                    executors.push(Arc::new(executor));
                    tracing::info!(
                        id = %spec.id,
                        endpoint = %spec.service_endpoint,
                        "Registered STT transform"
                    );
                }
                "llm" => {
                    let executor = crate::transform::llm::LlmExecutor::new(
                        spec.id.clone(),
                        source,
                        spec.service_endpoint.clone(),
                        &spec.params,
                    );
                    executors.push(Arc::new(executor));
                    tracing::info!(
                        id = %spec.id,
                        endpoint = %spec.service_endpoint,
                        "Registered LLM transform"
                    );
                }
                other => {
                    tracing::warn!(transform_id = %other, "Unknown transform type; skipping");
                }
            }
        }

        let transform_dag = match TransformDag::new(executors) {
            Ok(dag) => {
                tracing::info!(?dag, "Transform DAG constructed");
                Arc::new(dag)
            }
            Err(e) => {
                tracing::error!(error = %e, "Transform DAG has cycles, falling back to empty DAG");
                Arc::new(TransformDag::new(vec![]).expect("empty DAG"))
            }
        };

        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .map_err(|e| LifelogError::Other(e.into()))?;

        Ok(Server {
            db,
            postgres_pool,
            cas,
            config: Arc::new(RwLock::new(config.clone())),
            state: Arc::new(RwLock::new(system_state)),
            registered_collectors: Arc::new(RwLock::new(vec![])),
            policy: Arc::new(RwLock::new(ServerPolicy::new(policy_config))),
            transform_dag,
            http_client,
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
        let normalized = normalize_collector_id(&collector_name);
        for collector in collectors.iter() {
            if normalize_collector_id(&collector.id) == normalized {
                return true;
            }
        }
        false
    }

    async fn get_state(&self) -> SystemState {
        let mut state = self.state.read().await.clone();
        if let Some(server_state) = state.server_state.as_mut() {
            {
                let pool_status = self.postgres_pool.status();
                server_state.postgres_pool_enabled = true;
                server_state.postgres_pool_max_size = pool_status.max_size as u32;
                server_state.postgres_pool_size = pool_status.size as u32;
                server_state.postgres_pool_available = pool_status.available as u32;
                server_state.postgres_pool_waiting = pool_status.waiting as u32;

                if let Ok(client) = self.postgres_pool.get().await {
                    let count_sql = "\
                        SELECT COALESCE(SUM(cnt), 0)::BIGINT AS total FROM ( \
                            SELECT COUNT(*) AS cnt FROM screen_records \
                            UNION ALL SELECT COUNT(*) FROM browser_records \
                            UNION ALL SELECT COUNT(*) FROM ocr_records \
                            UNION ALL SELECT COUNT(*) FROM audio_records \
                            UNION ALL SELECT COUNT(*) FROM clipboard_records \
                            UNION ALL SELECT COUNT(*) FROM shell_history_records \
                            UNION ALL SELECT COUNT(*) FROM keystroke_records \
                            UNION ALL SELECT COUNT(*) FROM frames \
                        ) sub";
                    if let Ok(row) = client.query_one(count_sql, &[]).await {
                        let total: i64 = row.get(0);
                        server_state.total_frames_stored = total as u64;
                    }

                    let size_sql = "SELECT pg_database_size(current_database())::BIGINT";
                    if let Ok(row) = client.query_one(size_sql, &[]).await {
                        let size: i64 = row.get(0);
                        server_state.disk_usage_bytes = size as u64;
                    }
                }
            }
        }
        state
    }

    async fn get_config(&self) -> SystemConfig {
        let server_config = self.config.read().await.clone();
        let collectors = self.registered_collectors.read().await;
        let mut collector_configs = HashMap::new();
        for collector in collectors.iter() {
            if let Some(config) = &collector.latest_config {
                collector_configs.insert(collector.id.clone(), config.clone());
            }
        }

        SystemConfig {
            server: Some(server_config),
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

        let available_origins = self.get_available_origins_hybrid().await?;
        let scoped_origins: Vec<DataOrigin> = if query_msg.search_origins.is_empty() {
            available_origins.clone()
        } else {
            let mut resolved = Vec::new();
            let mut seen = HashSet::new();
            for s in &query_msg.search_origins {
                for o in self.resolve_search_origins(s, &available_origins).await {
                    let key = o.get_table_name();
                    if seen.insert(key) {
                        resolved.push(o);
                    }
                }
            }
            resolved
        };

        // LLQL (JSON) escape hatch: allow the UI to execute typed cross-modal queries
        // without changing the protobuf. Use `Query.text = ["llql:{...json...}"]`.
        if let Some(ast_query) = crate::query::llql::try_parse_llql(&query_msg.text)? {
            let default_window_ms = self.config.read().await.default_correlation_window_ms;
            let default_window =
                chrono::Duration::milliseconds(i64::try_from(default_window_ms).unwrap_or(30_000));

            let ast_query = crate::query::ast::Query {
                target: ast_query.target,
                filter: ast_query
                    .filter
                    .with_default_temporal_windows(default_window),
            };

            let plan = crate::query::planner::Planner::plan(&ast_query, &scoped_origins);
            let res = if crate::query::executor::plan_is_postgres_compatible(&plan) {
                crate::query::executor::execute_postgres(&self.postgres_pool, plan).await
            } else {
                crate::query::executor::execute(&self.db, plan).await
            };
            return res.map_err(|e| LifelogError::Database(format!("query execution failed: {e}")));
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
                // Avoid Surreal datetime parser overflow on extreme chrono bounds.
                // Use a wide, practical range as the "match all" fallback.
                (None, None) => {
                    let start = chrono::DateTime::from_timestamp(0, 0)
                        .unwrap_or(chrono::DateTime::<Utc>::MIN_UTC);
                    let end = chrono::DateTime::from_timestamp(4_102_444_800, 0)
                        .unwrap_or(chrono::DateTime::<Utc>::MAX_UTC); // 2100-01-01T00:00:00Z
                    crate::query::ast::Expression::TimeRange(start, end)
                }
            };

            let query = crate::query::ast::Query {
                target: crate::query::ast::StreamSelector::StreamId(table.clone()),
                filter,
            };

            // Pass the full catalog so temporal operators like WITHIN can resolve other streams.
            let plan = crate::query::planner::Planner::plan(&query, &available_origins);
            let query_result = if crate::query::executor::plan_is_postgres_compatible(&plan) {
                crate::query::executor::execute_postgres(&self.postgres_pool, plan).await
            } else {
                crate::query::executor::execute(&self.db, plan).await
            };
            match query_result {
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

        let available_origins = self.get_available_origins_hybrid().await?;

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

        #[derive(Debug)]
        struct PgScreenRow {
            uuid: String,
            t_canonical: Option<chrono::DateTime<Utc>>,
        }

        let screen_frames: Vec<(String, chrono::DateTime<Utc>)> = {
            let pool = &self.postgres_pool;
            if let Some(table) = Self::postgres_table_for_modality(&screen_origin.modality_name) {
                let collector_id =
                    Self::collector_id_from_origin(&screen_origin).ok_or_else(|| {
                        LifelogError::Validation {
                            field: "screen_origin".to_string(),
                            reason: "screen origin must include collector identity".to_string(),
                        }
                    })?;
                let sql = format!(
                        "SELECT id::text AS id, t_canonical::text AS t_canonical FROM {table} WHERE collector_id = $1 AND time_range && tstzrange($2::timestamptz, $3::timestamptz, '[)') ORDER BY lower(time_range) ASC LIMIT $4"
                    );
                let client = pool.get().await.map_err(|e| {
                    LifelogError::Database(format!("postgres pool get failed: {e}"))
                })?;
                let rows = client
                    .query(
                        sql.as_str(),
                        &[
                            &collector_id,
                            &start.to_rfc3339(),
                            &end.to_rfc3339(),
                            &(max_steps as i64),
                        ],
                    )
                    .await
                    .map_err(|e| {
                        LifelogError::Database(format!("replay postgres screen query failed: {e}"))
                    })?;
                rows.into_iter()
                    .map(|r| PgScreenRow {
                        uuid: r.get("id"),
                        t_canonical: r
                            .get::<_, Option<String>>("t_canonical")
                            .and_then(|ts| chrono::DateTime::parse_from_rfc3339(&ts).ok())
                            .map(|dt| dt.with_timezone(&Utc)),
                    })
                    .filter_map(|r| Some((r.uuid, r.t_canonical?)))
                    .collect()
            } else {
                let screen_table = screen_origin.get_table_name();
                let sql = format!(
                        "SELECT uuid, t_canonical FROM `{}` WHERE t_canonical >= d'{}' AND t_canonical < d'{}' ORDER BY t_canonical ASC LIMIT {};",
                        screen_table,
                        start.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                        end.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
                        max_steps
                    );
                let mut resp = self.db.query(sql).await.map_err(|e| {
                    LifelogError::Database(format!("replay screen query failed: {e}"))
                })?;
                let rows: Vec<ScreenRow> = resp.take(0).map_err(|e| {
                    LifelogError::Database(format!("replay screen take failed: {e}"))
                })?;
                rows.into_iter()
                    .filter_map(|r| Some((r.uuid, r.t_canonical?.0)))
                    .collect()
            }
        };

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

        #[derive(Debug)]
        struct PgCtxRow {
            uuid: String,
            t_canonical: Option<chrono::DateTime<Utc>>,
            t_end: Option<chrono::DateTime<Utc>>,
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
                {
                    let pool = &self.postgres_pool;
                    if let Some(table) = Self::postgres_table_for_modality(&origin.modality_name) {
                        let collector_id =
                            Self::collector_id_from_origin(&origin).ok_or_else(|| {
                                LifelogError::Validation {
                                    field: "context_origins".to_string(),
                                    reason: format!(
                                        "origin '{}' must include collector identity",
                                        origin.get_table_name()
                                    ),
                                }
                            })?;
                        let sql = format!(
                            "SELECT id::text AS id, t_canonical::text AS t_canonical, t_end::text AS t_end FROM {table} WHERE collector_id = $1 AND time_range && tstzrange($2::timestamptz, $3::timestamptz, '[]') ORDER BY lower(time_range) ASC LIMIT $4"
                        );
                        let client = pool.get().await.map_err(|e| {
                            LifelogError::Database(format!("postgres pool get failed: {e}"))
                        })?;
                        let rows = client
                            .query(
                                sql.as_str(),
                                &[
                                    &collector_id,
                                    &window_start.to_rfc3339(),
                                    &window_end.to_rfc3339(),
                                    &(per_origin_limit as i64),
                                ],
                            )
                            .await
                            .map_err(|e| {
                                LifelogError::Database(format!(
                                    "replay postgres context query failed for {table}: {e}"
                                ))
                            })?;
                        let pg_rows: Vec<PgCtxRow> = rows
                            .into_iter()
                            .map(|r| PgCtxRow {
                                uuid: r.get("id"),
                                t_canonical: r
                                    .get::<_, Option<String>>("t_canonical")
                                    .and_then(|ts| chrono::DateTime::parse_from_rfc3339(&ts).ok())
                                    .map(|dt| dt.with_timezone(&Utc)),
                                t_end: r
                                    .get::<_, Option<String>>("t_end")
                                    .and_then(|ts| chrono::DateTime::parse_from_rfc3339(&ts).ok())
                                    .map(|dt| dt.with_timezone(&Utc)),
                            })
                            .collect();
                        for r in pg_rows {
                            let Some(t0) = r.t_canonical else { continue };
                            let start = t0;
                            let end = r.t_end.unwrap_or(start);
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
                        continue;
                    }
                }

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

    async fn run_retention_once(
        &self,
    ) -> Result<crate::retention::RetentionRunSummary, LifelogError> {
        let policy = self.config.read().await.retention_policy_days.clone();
        prune_once(&self.db, &self.cas, &policy, Utc::now()).await
    }

    async fn apply_system_config(&self, system_config: SystemConfig) -> Result<(), LifelogError> {
        if let Some(new_server_config) = system_config.server {
            new_server_config.validate()?;
            let mut current = self.config.write().await;
            current.default_correlation_window_ms = new_server_config.default_correlation_window_ms;
            current.retention_policy_days = new_server_config.retention_policy_days;
        }

        if system_config.collectors.is_empty() {
            return Ok(());
        }

        let mut updates: Vec<(
            tokio::sync::mpsc::Sender<Result<ServerCommand, tonic::Status>>,
            CollectorConfig,
            String,
        )> = Vec::new();
        {
            let mut registered = self.registered_collectors.write().await;
            for (collector_id, new_cfg) in system_config.collectors {
                if let Some(collector) = registered.iter_mut().find(|c| c.id == collector_id) {
                    collector.latest_config = Some(new_cfg.clone());
                    updates.push((collector.command_tx.clone(), new_cfg, collector.id.clone()));
                }
            }
        }

        for (tx, cfg, collector_id) in updates {
            let payload = serde_json::to_string(&cfg).map_err(|e| LifelogError::Validation {
                field: "collector_config".to_string(),
                reason: format!("failed to serialize update payload: {e}"),
            })?;
            let cmd = ServerCommand {
                r#type: CommandType::UpdateConfig as i32,
                payload,
            };
            if tx.send(Ok(cmd)).await.is_err() {
                tracing::warn!(
                    collector_id = %collector_id,
                    "collector config update command was not delivered"
                );
            }
        }

        Ok(())
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
                let pool_clone = self.postgres_pool.clone();
                let dag = self.transform_dag.clone();
                let http = self.http_client.clone();

                tokio::spawn(async move {
                    let watermarks: std::sync::Arc<
                        dyn crate::transform::watermark::WatermarkStore,
                    > = std::sync::Arc::new(
                        crate::transform::watermark::PostgresWatermarkStore::new(
                            pool_clone.clone(),
                        ),
                    );

                    let worker = crate::transform::worker::PipelineWorker::new(
                        dag,
                        watermarks,
                        db_connection,
                        Some(pool_clone),
                        cas_clone,
                        http,
                        50,
                    );

                    if let Err(e) = worker.poll_once().await {
                        tracing::error!(error = %e, "Transform pipeline error");
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
