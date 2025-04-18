// Main server application entry point and configuration
use ::config as config_crate;
use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::{get, middleware, post, put, web, App, Error, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use lifelog_server::auth::{Claims, JwtAuth, UserStore};
use lifelog_server::auth_handlers::{self, AuthState};
use lifelog_server::cors::cors_config;
use lifelog_server::db::{init_db, LoggerDb};
use lifelog_server::error::ApiError;
use lifelog_server::handlers::get_logger_data as surreal_get_logger_data;
use lifelog_server::handlers::{
    get_logger_count, insert_camera_frame, insert_microphone_recording, insert_process_data,
    insert_screen_capture,
};
use rusqlite::types::ValueRef;
use rusqlite::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use tokio::process::Command as TokioCommand;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use ::config::{
    CameraConfig, Config, HyprlandConfig, MicrophoneConfig, ProcessesConfig, ScreenConfig,
};

fn load_env_config() {
    dotenv().ok();
}

fn setup_logging() {
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(format!(
            "lifelog_server={},actix_web=info,actix_http=info",
            log_level
        )))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Logging initialized at {} level", log_level);
}

enum DataSourceKind {
    Hyprland {
        path_to_db: PathBuf,
    },
    Screen {
        path_to_db: PathBuf,
        path_to_dir: PathBuf,
    },
    Processes {
        path_to_db: PathBuf,
    },
    Microphone {
        path_to_db: PathBuf,
        path_to_dir: PathBuf,
    },
    Camera {
        path_to_db: PathBuf,
        path_to_dir: PathBuf,
    },
}

enum ConfigKind {
    Hyprland(HyprlandConfig),
    Screen(ScreenConfig),
    Processes(ProcessesConfig),
    Microphone(MicrophoneConfig),
    Camera(CameraConfig),
}

struct DataSource {
    kind: DataSourceKind,
    conn: Connection,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggerConfig {
    enabled: bool,
    interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggerStatus {
    enabled: bool,
    running: bool,
}

#[derive(Serialize, Deserialize)]
struct PaginationParams {
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct QueryParams {
    start_time: Option<f64>,
    end_time: Option<f64>,
    limit: Option<u32>,
    filter: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    status: String,
    data: T,
}

struct AppState {
    config: Arc<Mutex<config_crate::Config>>,
    loggers: HashMap<String, Arc<Mutex<LoggerStatus>>>,
}

fn data_source_kind_to_table_names(kind: &DataSourceKind) -> Vec<String> {
    match kind {
        DataSourceKind::Hyprland { .. } => vec![
            "clients",
            "devices",
            "cursor_positions",
            "activeworkspace",
            "workspaces",
            "monitors",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect(),
        DataSourceKind::Screen { .. } => vec!["screen".to_string()],
        DataSourceKind::Processes { .. } => vec!["processes".to_string()],
        DataSourceKind::Microphone { .. } => vec!["microphone".to_string()],
        DataSourceKind::Camera { .. } => vec!["camera".to_string()],
    }
}

fn execute_query_on_table(
    conn: &Connection,
    table_name: &str,
    query: &str,
) -> Result<Vec<serde_json::Value>> {
    let sql = format!("SELECT * FROM {} WHERE {}", table_name, query);

    let mut stmt = conn.prepare(&sql)?;
    let column_names: Vec<String> = stmt
        .column_names()
        .iter()
        .map(|name| name.to_string())
        .collect();

    let rows = stmt.query_map([], |row| {
        let mut obj = serde_json::Map::new();

        for (i, name) in column_names.iter().enumerate() {
            let value: serde_json::Value = match row.get_ref(i)? {
                ValueRef::Null => serde_json::Value::Null,
                ValueRef::Integer(i) => serde_json::Value::Number(serde_json::Number::from(i)),
                ValueRef::Real(f) => {
                    if let Some(n) = serde_json::Number::from_f64(f) {
                        serde_json::Value::Number(n)
                    } else {
                        serde_json::Value::Null
                    }
                }
                ValueRef::Text(t) => {
                    serde_json::Value::String(String::from_utf8_lossy(t).to_string())
                }
                ValueRef::Blob(b) => {
                    let base64 = base64::encode(b);
                    serde_json::Value::String(format!("BLOB:{}", base64))
                }
            };

            obj.insert(name.clone(), value);
        }

        Ok(serde_json::Value::Object(obj))
    })?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row?);
    }

    Ok(results)
}
// TODO: Add support for CRON-like scheduling for:
//       - Data processing
//       - Synching between data sources (cloud data sources, browser history, cliphistory)
// TODO: Add support for queries for the database
// TODO: Add support for natural language to query
//

fn main() {

    // Passes test cases 🤓
    //let config = load_config();
    //
    //let matches = Command::new("lifelog-server")
    //    .version("0.1.0")
    //    .author("Matt Handzel <handzelmatthew@gmail.com>")
    //    .about("Lifelog Server")
    //    .arg(Arg::new("query").short('q').long("query").takes_value(true))
    //    .get_matches();
    //
    //let data_sources: Vec<_> = [
    //    ConfigKind::Hyprland(config.hyprland),
    //    ConfigKind::Screen(config.screen),
    //    ConfigKind::Processes(config.processes),
    //]
    //.iter()
    //.map(|_config| -> DataSource {
    //    match _config {
    //        // Bruh why is this so verbose 😭
    //        ConfigKind::Hyprland(HyprlandConfig {
    //            enabled,
    //            interval,
    //            output_dir,
    //            log_clients,
    //            log_activewindow,
    //            log_workspace,
    //            log_active_monitor,
    //            log_devices,
    //        }) => DataSource {
    //            kind: DataSourceKind::Hyprland {
    //                path_to_db: output_dir.join("hyprland.db"),
    //            },
    //            conn: setup::setup_hyprland_db(output_dir).expect("Failed to setup hyprland db"),
    //        },
    //        ConfigKind::Screen(ScreenConfig {
    //            enabled,
    //            interval,
    //            output_dir,
    //            program,
    //            timestamp_format,
    //        }) => DataSource {
    //            kind: DataSourceKind::Screen {
    //                path_to_dir: output_dir.clone(),
    //                path_to_db: output_dir.join("screen.db"),
    //            },
    //            conn: setup::setup_screen_db(output_dir).expect("Failed to setup screen db"),
    //        },
    //        ConfigKind::Processes(ProcessesConfig {
    //            enabled,
    //            interval,
    //            output_dir,
    //        }) => DataSource {
    //            kind: DataSourceKind::Processes {
    //                path_to_db: output_dir.join("processes.db"),
    //            },
    //            conn: setup::setup_process_db(output_dir).expect("Failed to setup processes db"),
    //        },
    //        ConfigKind::Microphone(MicrophoneConfig { output_dir, .. }) => DataSource {
    //            kind: DataSourceKind::Microphone {
    //                path_to_dir: output_dir.clone(),
    //                path_to_db: output_dir.join("microphone.db"),
    //            },
    //            conn: setup::setup_microphone_db(output_dir)
    //                .expect("Failed to setup microphone db"),
    //        },
    //    }
    //})
    //.collect();
    //
    ////setup_embeddings_db(&config.server).expect("Failed to setup embeddings db");
    //
    //println!("Data sources opened");
    //
    //if let Some(query) = matches.value_of("query") {
    //    println!("Query: {}", query);
    //
    //    for data_source in data_sources {
    //        let table_names = data_source_kind_to_table_names(data_source.kind);
    //
    //        for table_name in table_names {
    //            println!("Executing query on table: {}", table_name);
    //            if let Err(e) = execute_query_on_table(&data_source.conn, &table_name, query) {
    //                eprintln!("Error executing query on table {}: {}", table_name, e);
    //            }
    //        }
    //    }
    //}
}
