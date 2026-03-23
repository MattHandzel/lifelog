use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDate, Utc};
use clap::{Parser, ValueEnum};
use deadpool_postgres::Pool;
use serde::Serialize;
use serde_json::Value;
use std::io;
use tokio_postgres::NoTls;

#[derive(Parser)]
#[command(
    name = "lifelog-export",
    about = "Export lifelog frames as JSON or CSV"
)]
struct Cli {
    #[arg(short, long, help = "Output format")]
    format: OutputFormat,

    #[arg(short, long, help = "Filter by modality (e.g. screen, browser, ocr)")]
    modality: Option<String>,

    #[arg(long, help = "Start date (YYYY-MM-DD or ISO 8601)")]
    start: Option<String>,

    #[arg(long, help = "End date (YYYY-MM-DD or ISO 8601)")]
    end: Option<String>,

    #[arg(short, long, help = "Full-text search query")]
    query: Option<String>,

    #[arg(short, long, default_value = "1000", help = "Maximum number of frames")]
    limit: i64,

    #[arg(long, help = "Output file (default: stdout)")]
    output: Option<String>,

    #[arg(
        long,
        help = "PostgreSQL connection string (default: from config or env)"
    )]
    database_url: Option<String>,

    #[arg(long, help = "Include blob hashes in output")]
    include_blobs: bool,

    #[arg(
        long,
        help = "Flatten payload fields into top-level columns (CSV only)"
    )]
    flatten: bool,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Csv,
    Jsonl,
}

#[derive(Serialize)]
struct ExportFrame {
    id: String,
    collector_id: String,
    modality: String,
    timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    end_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blob_hash: Option<String>,
    payload: Value,
}

fn resolve_database_url(explicit: Option<&str>) -> Result<String> {
    if let Some(url) = explicit {
        return Ok(url.to_string());
    }
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

fn parse_date_bound(s: &str, is_end: bool) -> Result<String> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.to_rfc3339());
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let d = if is_end {
            d + chrono::Duration::days(1)
        } else {
            d
        };
        let dt = d
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow!("bad date"))?
            .and_utc();
        return Ok(dt.to_rfc3339());
    }
    Err(anyhow!("Cannot parse date: {s}"))
}

async fn fetch_frames(pool: &Pool, cli: &Cli) -> Result<Vec<ExportFrame>> {
    let mut conditions: Vec<String> = Vec::new();
    let mut param_idx = 1u32;
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync + Send>> = Vec::new();

    if let Some(ref q) = cli.query {
        if !q.trim().is_empty() {
            conditions.push(format!(
                "search_doc @@ websearch_to_tsquery('english', ${param_idx})"
            ));
            param_idx += 1;
            params.push(Box::new(q.clone()));
        }
    }

    if let Some(ref m) = cli.modality {
        conditions.push(format!("modality = ${param_idx}"));
        param_idx += 1;
        params.push(Box::new(m.clone()));
    }

    if let Some(ref sd) = cli.start {
        conditions.push(format!("t_canonical >= ${param_idx}::timestamptz"));
        param_idx += 1;
        params.push(Box::new(parse_date_bound(sd, false)?));
    }

    if let Some(ref ed) = cli.end {
        conditions.push(format!("t_canonical < ${param_idx}::timestamptz"));
        let _ = param_idx;
        params.push(Box::new(parse_date_bound(ed, true)?));
    }

    let where_clause = if conditions.is_empty() {
        "TRUE".to_string()
    } else {
        conditions.join(" AND ")
    };

    let limit = cli.limit.min(100_000);
    let sql = format!(
        "SELECT id::text, collector_id, modality, t_canonical, t_end, blob_hash, payload
         FROM frames
         WHERE {where_clause}
         ORDER BY t_canonical ASC
         LIMIT {limit}"
    );

    let client = pool.get().await?;
    let param_refs: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = params
        .iter()
        .map(|p| p.as_ref() as &(dyn tokio_postgres::types::ToSql + Sync))
        .collect();
    let rows = client.query(&sql, &param_refs).await?;

    let frames: Vec<ExportFrame> = rows
        .iter()
        .map(|row| {
            let t_canonical: DateTime<Utc> = row.get("t_canonical");
            let t_end: Option<DateTime<Utc>> = row.get("t_end");
            let blob_hash: Option<String> = row.get("blob_hash");

            ExportFrame {
                id: row.get("id"),
                collector_id: row.get("collector_id"),
                modality: row.get("modality"),
                timestamp: t_canonical.to_rfc3339(),
                end_time: t_end.map(|t| t.to_rfc3339()),
                blob_hash: if cli.include_blobs { blob_hash } else { None },
                payload: row.get("payload"),
            }
        })
        .collect();

    Ok(frames)
}

fn write_json(frames: &[ExportFrame], writer: &mut dyn io::Write) -> Result<()> {
    serde_json::to_writer_pretty(&mut *writer, frames)?;
    writeln!(writer)?;
    Ok(())
}

fn write_jsonl(frames: &[ExportFrame], writer: &mut dyn io::Write) -> Result<()> {
    for frame in frames {
        serde_json::to_writer(&mut *writer, frame)?;
        writeln!(writer)?;
    }
    Ok(())
}

fn write_csv(frames: &[ExportFrame], writer: &mut dyn io::Write, flatten: bool) -> Result<()> {
    let mut wtr = csv::Writer::from_writer(writer);

    if flatten {
        let payload_keys = collect_payload_keys(frames);

        let mut header = vec![
            "id".to_string(),
            "collector_id".to_string(),
            "modality".to_string(),
            "timestamp".to_string(),
            "end_time".to_string(),
            "blob_hash".to_string(),
        ];
        for key in &payload_keys {
            header.push(format!("payload.{key}"));
        }
        wtr.write_record(&header)?;

        for frame in frames {
            let mut record = vec![
                frame.id.clone(),
                frame.collector_id.clone(),
                frame.modality.clone(),
                frame.timestamp.clone(),
                frame.end_time.clone().unwrap_or_default(),
                frame.blob_hash.clone().unwrap_or_default(),
            ];
            for key in &payload_keys {
                let val = frame
                    .payload
                    .get(key)
                    .map(|v| match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    })
                    .unwrap_or_default();
                record.push(val);
            }
            wtr.write_record(&record)?;
        }
    } else {
        wtr.write_record([
            "id",
            "collector_id",
            "modality",
            "timestamp",
            "end_time",
            "blob_hash",
            "payload",
        ])?;
        for frame in frames {
            wtr.write_record(&[
                &frame.id,
                &frame.collector_id,
                &frame.modality,
                &frame.timestamp,
                frame.end_time.as_deref().unwrap_or(""),
                frame.blob_hash.as_deref().unwrap_or(""),
                &serde_json::to_string(&frame.payload).unwrap_or_default(),
            ])?;
        }
    }

    wtr.flush()?;
    Ok(())
}

fn collect_payload_keys(frames: &[ExportFrame]) -> Vec<String> {
    let mut keys = std::collections::BTreeSet::new();
    for frame in frames {
        if let Value::Object(map) = &frame.payload {
            for key in map.keys() {
                if let Some(Value::Object(_) | Value::Array(_)) = map.get(key) {
                    continue;
                }
                keys.insert(key.clone());
            }
        }
    }
    keys.into_iter().collect()
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

    let cli = Cli::parse();
    let database_url = resolve_database_url(cli.database_url.as_deref())?;
    let pool = create_pool(&database_url).await?;

    eprintln!("Fetching frames...");
    let frames = fetch_frames(&pool, &cli).await?;
    eprintln!("Exporting {} frames...", frames.len());

    let mut writer: Box<dyn io::Write> = if let Some(ref path) = cli.output {
        Box::new(std::fs::File::create(path)?)
    } else {
        Box::new(io::stdout().lock())
    };

    match cli.format {
        OutputFormat::Json => write_json(&frames, &mut writer)?,
        OutputFormat::Jsonl => write_jsonl(&frames, &mut writer)?,
        OutputFormat::Csv => write_csv(&frames, &mut writer, cli.flatten)?,
    }

    if let Some(ref path) = cli.output {
        eprintln!("Exported {} frames to {path}", frames.len());
    }

    Ok(())
}
