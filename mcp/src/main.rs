use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, Utc};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write as _};
use tokio_postgres::NoTls;
use tracing::debug;

const SERVER_NAME: &str = "lifelog-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i64,
    message: String,
}

impl JsonRpcResponse {
    fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    fn error(id: Option<Value>, code: i64, message: String) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError { code, message }),
        }
    }
}

fn resolve_database_url() -> Result<String> {
    if let Ok(url) = std::env::var("LIFELOG_POSTGRES_INGEST_URL") {
        if !url.trim().is_empty() {
            return Ok(url);
        }
    }

    let config_paths = [
        std::env::var("HOME")
            .ok()
            .map(|h| std::path::PathBuf::from(h).join(".config/lifelog/lifelog-config.toml")),
        Some(std::path::PathBuf::from("lifelog-config.toml")),
    ];

    for path in config_paths.iter().flatten() {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(table) = contents.parse::<toml::Table>() {
                if let Some(server) = table.get("server").and_then(|v| v.as_table()) {
                    if let Some(url) = server
                        .get("postgresUrl")
                        .or_else(|| server.get("postgres_url"))
                        .and_then(|v| v.as_str())
                    {
                        return Ok(url.to_string());
                    }
                }
            }
        }
    }

    Ok("host=/run/postgresql dbname=lifelog".into())
}

async fn create_pool(database_url: &str) -> Result<Pool> {
    let cfg: tokio_postgres::Config = database_url.parse()?;
    let mgr_cfg = deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Fast,
    };
    let mgr = deadpool_postgres::Manager::from_config(cfg, NoTls, mgr_cfg);
    let pool = Pool::builder(mgr).max_size(4).build()?;
    let _ = pool.get().await?;
    Ok(pool)
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "search_frames",
            "description": "Search lifelog frames by text query, modality, and/or date range. Returns matching frames with metadata and text content.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Full-text search query (uses PostgreSQL websearch syntax)"
                    },
                    "modality": {
                        "type": "string",
                        "description": "Filter by data modality (e.g. screen, browser, audio, clipboard, shell_history, keystroke, window_activity, process, weather, ocr, transcription)"
                    },
                    "start_date": {
                        "type": "string",
                        "description": "Start of date range (ISO 8601 or YYYY-MM-DD)"
                    },
                    "end_date": {
                        "type": "string",
                        "description": "End of date range (ISO 8601 or YYYY-MM-DD)"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results (default 20, max 100)"
                    }
                }
            }
        },
        {
            "name": "get_frame",
            "description": "Get a single lifelog frame by its UUID. Returns full metadata and payload.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "UUID of the frame"
                    }
                },
                "required": ["id"]
            }
        },
        {
            "name": "list_modalities",
            "description": "List all data modalities present in the lifelog database with frame counts and date ranges.",
            "inputSchema": {
                "type": "object",
                "properties": {}
            }
        },
        {
            "name": "get_timeline",
            "description": "Get a timeline of activity for a specific date, grouped by hour. Shows what modalities captured data and key events.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "date": {
                        "type": "string",
                        "description": "Date to get timeline for (YYYY-MM-DD, defaults to today)"
                    },
                    "modality": {
                        "type": "string",
                        "description": "Optional: filter timeline to a specific modality"
                    }
                }
            }
        }
    ])
}

async fn handle_tool_call(pool: &Pool, name: &str, args: &Value) -> Result<Value> {
    match name {
        "search_frames" => tool_search_frames(pool, args).await,
        "get_frame" => tool_get_frame(pool, args).await,
        "list_modalities" => tool_list_modalities(pool).await,
        "get_timeline" => tool_get_timeline(pool, args).await,
        _ => Err(anyhow!("Unknown tool: {name}")),
    }
}

async fn tool_search_frames(pool: &Pool, args: &Value) -> Result<Value> {
    let query = args.get("query").and_then(|v| v.as_str());
    let modality = args.get("modality").and_then(|v| v.as_str());
    let start_date = args.get("start_date").and_then(|v| v.as_str());
    let end_date = args.get("end_date").and_then(|v| v.as_str());
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(20)
        .min(100) as i64;

    let mut conditions: Vec<String> = Vec::new();
    let mut param_idx = 1u32;

    if let Some(q) = query {
        if !q.trim().is_empty() {
            conditions.push(format!(
                "search_doc @@ websearch_to_tsquery('english', ${param_idx})"
            ));
            param_idx += 1;
        }
    }

    if modality.is_some() {
        conditions.push(format!("modality = ${param_idx}"));
        param_idx += 1;
    }

    if start_date.is_some() {
        conditions.push(format!("t_canonical >= ${param_idx}::timestamptz"));
        param_idx += 1;
    }

    if end_date.is_some() {
        conditions.push(format!("t_canonical < ${param_idx}::timestamptz"));
        let _ = param_idx;
    }

    let where_clause = if conditions.is_empty() {
        "TRUE".to_string()
    } else {
        conditions.join(" AND ")
    };

    let sql = format!(
        "SELECT id::text, collector_id, modality, t_canonical, t_end, payload, blob_hash
         FROM frames
         WHERE {where_clause}
         ORDER BY t_canonical DESC
         LIMIT {limit}"
    );

    debug!(sql = %sql, "search_frames query");

    let client = pool.get().await?;

    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();
    if let Some(q) = query {
        if !q.trim().is_empty() {
            params.push(Box::new(q.to_string()));
        }
    }
    if let Some(m) = modality {
        params.push(Box::new(m.to_string()));
    }
    if let Some(sd) = start_date {
        params.push(Box::new(parse_date_start(sd)?));
    }
    if let Some(ed) = end_date {
        params.push(Box::new(parse_date_end(ed)?));
    }

    let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();

    let rows = client.query(&sql, &param_refs).await?;

    let results: Vec<Value> = rows
        .iter()
        .map(|row| {
            let id: String = row.get("id");
            let collector_id: String = row.get("collector_id");
            let modality: String = row.get("modality");
            let t_canonical: DateTime<Utc> = row.get("t_canonical");
            let t_end: Option<DateTime<Utc>> = row.get("t_end");
            let payload: Value = row.get("payload");
            let blob_hash: Option<String> = row.get("blob_hash");

            let mut result = json!({
                "id": id,
                "collector_id": collector_id,
                "modality": modality,
                "timestamp": t_canonical.to_rfc3339(),
                "payload": payload,
            });

            if let Some(end) = t_end {
                result["end_time"] = json!(end.to_rfc3339());
            }
            if let Some(hash) = blob_hash {
                result["has_blob"] = json!(true);
                result["blob_hash"] = json!(hash);
            }

            result
        })
        .collect();

    Ok(json!({
        "count": results.len(),
        "frames": results,
    }))
}

async fn tool_get_frame(pool: &Pool, args: &Value) -> Result<Value> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("'id' parameter is required"))?;

    let uuid: uuid::Uuid = id.parse().map_err(|e| anyhow!("Invalid UUID: {e}"))?;

    let client = pool.get().await?;
    let row = client
        .query_opt(
            "SELECT id::text, collector_id, stream_id, modality, t_device, t_ingest,
                    t_canonical, t_end, time_quality, blob_hash, blob_size,
                    indexed, source_frame_id::text, payload
             FROM frames WHERE id = $1",
            &[&uuid],
        )
        .await?
        .ok_or_else(|| anyhow!("Frame not found: {id}"))?;

    let payload: Value = row.get("payload");
    let source_frame_id: Option<String> = row.get("source_frame_id");

    Ok(json!({
        "id": row.get::<_, String>("id"),
        "collector_id": row.get::<_, String>("collector_id"),
        "stream_id": row.get::<_, String>("stream_id"),
        "modality": row.get::<_, String>("modality"),
        "t_device": row.get::<_, Option<DateTime<Utc>>>("t_device").map(|t| t.to_rfc3339()),
        "t_ingest": row.get::<_, DateTime<Utc>>("t_ingest").to_rfc3339(),
        "t_canonical": row.get::<_, DateTime<Utc>>("t_canonical").to_rfc3339(),
        "t_end": row.get::<_, Option<DateTime<Utc>>>("t_end").map(|t| t.to_rfc3339()),
        "time_quality": row.get::<_, String>("time_quality"),
        "blob_hash": row.get::<_, Option<String>>("blob_hash"),
        "blob_size": row.get::<_, Option<i32>>("blob_size"),
        "indexed": row.get::<_, bool>("indexed"),
        "source_frame_id": source_frame_id,
        "payload": payload,
    }))
}

async fn tool_list_modalities(pool: &Pool) -> Result<Value> {
    let client = pool.get().await?;
    let rows = client
        .query(
            "SELECT modality,
                    COUNT(*) as frame_count,
                    MIN(t_canonical) as earliest,
                    MAX(t_canonical) as latest
             FROM frames
             GROUP BY modality
             ORDER BY frame_count DESC",
            &[],
        )
        .await?;

    let modalities: Vec<Value> = rows
        .iter()
        .map(|row| {
            let modality: String = row.get("modality");
            let count: i64 = row.get("frame_count");
            let earliest: DateTime<Utc> = row.get("earliest");
            let latest: DateTime<Utc> = row.get("latest");

            json!({
                "modality": modality,
                "frame_count": count,
                "earliest": earliest.to_rfc3339(),
                "latest": latest.to_rfc3339(),
            })
        })
        .collect();

    Ok(json!({
        "modalities": modalities,
        "total_modalities": modalities.len(),
    }))
}

async fn tool_get_timeline(pool: &Pool, args: &Value) -> Result<Value> {
    let date_str = args.get("date").and_then(|v| v.as_str());
    let modality = args.get("modality").and_then(|v| v.as_str());

    let date = if let Some(ds) = date_str {
        NaiveDate::parse_from_str(ds, "%Y-%m-%d")
            .map_err(|e| anyhow!("Invalid date format (use YYYY-MM-DD): {e}"))?
    } else {
        Utc::now().date_naive()
    };

    let start = date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| anyhow!("Invalid date"))?
        .and_utc();
    let end = (date + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| anyhow!("Invalid date"))?
        .and_utc();

    let (sql, params): (
        String,
        Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>>,
    ) = if let Some(m) = modality {
        (
            "SELECT date_trunc('hour', t_canonical) as hour,
                        modality,
                        COUNT(*) as frame_count,
                        MIN(payload->>'text') as sample_text,
                        MIN(payload->>'title') as sample_title
                 FROM frames
                 WHERE t_canonical >= $1 AND t_canonical < $2 AND modality = $3
                 GROUP BY hour, modality
                 ORDER BY hour, modality"
                .into(),
            vec![Box::new(start), Box::new(end), Box::new(m.to_string())],
        )
    } else {
        (
            "SELECT date_trunc('hour', t_canonical) as hour,
                        modality,
                        COUNT(*) as frame_count,
                        MIN(payload->>'text') as sample_text,
                        MIN(payload->>'title') as sample_title
                 FROM frames
                 WHERE t_canonical >= $1 AND t_canonical < $2
                 GROUP BY hour, modality
                 ORDER BY hour, modality"
                .into(),
            vec![Box::new(start), Box::new(end)],
        )
    };

    let client = pool.get().await?;
    let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();
    let rows = client.query(&sql, &param_refs).await?;

    let entries: Vec<Value> = rows
        .iter()
        .map(|row| {
            let hour: DateTime<Utc> = row.get("hour");
            let modality: String = row.get("modality");
            let count: i64 = row.get("frame_count");
            let sample_text: Option<String> = row.get("sample_text");
            let sample_title: Option<String> = row.get("sample_title");

            let mut entry = json!({
                "hour": hour.format("%H:%M").to_string(),
                "modality": modality,
                "frame_count": count,
            });

            if let Some(text) = sample_text {
                let truncated = if text.len() > 200 {
                    format!("{}...", &text[..200])
                } else {
                    text
                };
                entry["sample_text"] = json!(truncated);
            }
            if let Some(title) = sample_title {
                entry["sample_title"] = json!(title);
            }

            entry
        })
        .collect();

    let total_frames: i64 = entries
        .iter()
        .filter_map(|e| e.get("frame_count").and_then(|v| v.as_i64()))
        .sum();

    Ok(json!({
        "date": date.to_string(),
        "total_frames": total_frames,
        "timeline": entries,
    }))
}

fn parse_date_start(s: &str) -> Result<String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.to_rfc3339());
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = d
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow!("bad date"))?
            .and_utc();
        return Ok(dt.to_rfc3339());
    }
    Err(anyhow!("Cannot parse date: {s}"))
}

fn parse_date_end(s: &str) -> Result<String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.to_rfc3339());
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = (d + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow!("bad date"))?
            .and_utc();
        return Ok(dt.to_rfc3339());
    }
    Err(anyhow!("Cannot parse date: {s}"))
}

async fn handle_request(pool: &Pool, req: JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.clone();

    match req.method.as_str() {
        "initialize" => JsonRpcResponse::success(
            id,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": SERVER_NAME,
                    "version": SERVER_VERSION
                }
            }),
        ),

        "notifications/initialized" | "notifications/cancelled" => {
            JsonRpcResponse::success(id, json!(null))
        }

        "tools/list" => JsonRpcResponse::success(
            id,
            json!({
                "tools": tool_definitions()
            }),
        ),

        "tools/call" => {
            let tool_name = req
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = req.params.get("arguments").cloned().unwrap_or(json!({}));

            match handle_tool_call(pool, tool_name, &arguments).await {
                Ok(result) => {
                    let text = serde_json::to_string_pretty(&result).unwrap_or_default();
                    JsonRpcResponse::success(
                        id,
                        json!({
                            "content": [{
                                "type": "text",
                                "text": text,
                            }]
                        }),
                    )
                }
                Err(e) => JsonRpcResponse::success(
                    id,
                    json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Error: {e}"),
                        }],
                        "isError": true,
                    }),
                ),
            }
        }

        "ping" => JsonRpcResponse::success(id, json!({})),

        _ => JsonRpcResponse::error(id, -32601, format!("Method not found: {}", req.method)),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .with_writer(io::stderr)
        .init();

    let database_url = resolve_database_url()?;
    tracing::info!("Connecting to PostgreSQL...");
    let pool = create_pool(&database_url).await?;
    tracing::info!("Connected. MCP server ready on stdio.");

    let stdin = io::stdin();
    let stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::error(None, -32700, format!("Parse error: {e}"));
                let mut out = stdout.lock();
                serde_json::to_writer(&mut out, &resp)?;
                out.write_all(b"\n")?;
                out.flush()?;
                continue;
            }
        };

        if req.jsonrpc != "2.0" {
            let resp = JsonRpcResponse::error(req.id, -32600, "Invalid JSON-RPC version".into());
            let mut out = stdout.lock();
            serde_json::to_writer(&mut out, &resp)?;
            out.write_all(b"\n")?;
            out.flush()?;
            continue;
        }

        let is_notification = req.id.is_none();
        let resp = handle_request(&pool, req).await;

        if !is_notification || resp.error.is_some() {
            let mut out = stdout.lock();
            serde_json::to_writer(&mut out, &resp)?;
            out.write_all(b"\n")?;
            out.flush()?;
        }
    }

    Ok(())
}
