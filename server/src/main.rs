// Main server application entry point and configuration
use rusqlite::*;
use ::config as config_crate;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware, get, post, put, Error};
use actix_web::middleware::Logger;
use actix_files::{Files, NamedFile};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::env;
use dotenv::dotenv;
use tokio::process::Command as TokioCommand;
use tracing_actix_web::TracingLogger;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use serde_json::json;
use rusqlite::types::ValueRef;
use std::collections::HashMap;
use lifelog_server::auth::{Claims, JwtAuth, UserStore};
use lifelog_server::auth_handlers::{self, AuthState};
use lifelog_server::error::ApiError;
use lifelog_server::cors::cors_config;
use lifelog_server::db::{init_db, LoggerDb};
use lifelog_server::handlers::{get_logger_data as surreal_get_logger_data};
use lifelog_server::handlers::{
    insert_camera_frame, insert_screen_capture, 
    insert_microphone_recording, insert_process_data,
    get_logger_count,
};

use ::config::{Config, HyprlandConfig, ScreenConfig, ProcessesConfig, MicrophoneConfig, CameraConfig};

fn load_env_config() {
    dotenv().ok();
}

fn setup_logging() {
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            format!("lifelog_server={},actix_web=info,actix_http=info", log_level)
        ))
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

fn execute_query_on_table(conn: &Connection, table_name: &str, query: &str) -> Result<Vec<serde_json::Value>> {
    let sql = format!("SELECT * FROM {} WHERE {}", table_name, query);

    let mut stmt = conn.prepare(&sql)?;
    let column_names: Vec<String> = stmt.column_names().iter().map(|name| name.to_string()).collect();
    
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
                },
                ValueRef::Text(t) => serde_json::Value::String(String::from_utf8_lossy(t).to_string()),
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

#[get("/api/health")]
async fn health_check() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().json(json!({
        "status": "ok",
        "message": "Server is running"
    })))
}

#[post("/api/loggers/camera/capture")]
async fn capture_camera_frame(
    data: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let config = data.config.lock().unwrap();
    
    if !config.camera.enabled {
        return Err(ApiError::BadRequest("Camera is not enabled in settings".to_string()));
    }
    
    let output_dir = PathBuf::from(&config.camera.output_dir);
    if !output_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&output_dir) {
            return Err(ApiError::InternalServerError(format!("Failed to create output directory: {}", e)));
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        let timestamp = chrono::Local::now().timestamp() as f64;
        let filename = format!("camera_{}.jpg", timestamp);
        let output_path = output_dir.join(&filename);
        
        let result = Command::new("imagesnap")
            .arg("-w")
            .arg("1")
            .arg(output_path.to_str().unwrap())
            .output();
            
        match result {
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                
                if !output.status.success() || stderr.contains("error") {
                    return Err(ApiError::InternalServerError(format!("Failed to capture image: {}", stderr)));
                }
                
                let db_path = output_dir.join("camera.db");
                match Connection::open(&db_path) {
                    Ok(conn) => {
                        let result = conn.execute(
                            "INSERT INTO frames (timestamp, path) VALUES (?, ?)",
                            params![timestamp, filename],
                        );
                        
                        if let Err(e) = result {
                            return Err(ApiError::DatabaseError(format!("Failed to update database: {}", e)));
                        }
                    },
                    Err(e) => {
                        return Err(ApiError::DatabaseError(format!("Failed to open database: {}", e)));
                    }
                }
                
                Ok(HttpResponse::Ok().json(json!({
                    "status": "captured",
                    "path": filename,
                    "timestamp": timestamp
                })))
            },
            Err(e) => {
                Err(ApiError::InternalServerError(format!("Failed to execute capture command: {}", e)))
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    {
        Err(ApiError::InternalServerError("Camera capture not implemented for Linux".to_string()))
    }
    
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        Err(ApiError::InternalServerError("Camera capture not implemented for this platform".to_string()))
    }
}

#[get("/api/files/{logger_type}/{filename:.*}")]
async fn serve_file(
    params: web::Path<(String, String)>,
    data: web::Data<AppState>,
) -> Result<NamedFile, ApiError> {
    let (logger_type, filename) = params.into_inner();
    let config = data.config.lock().unwrap();
    
    let base_dir = match logger_type.as_str() {
        "screen" => &config.screen.output_dir,
        "camera" => &config.camera.output_dir,
        "microphone" => &config.microphone.output_dir,
        _ => return Err(ApiError::NotFound("Logger type not found".to_string())),
    };
    
    let file_path = PathBuf::from(base_dir).join(&filename);
    
    if !file_path.exists() {
        return Err(ApiError::NotFound(format!("File not found: {}", filename)));
    }
    
    match NamedFile::open(file_path) {
        Ok(file) => Ok(file),
        Err(e) => Err(ApiError::InternalServerError(format!("Failed to open file: {}", e))),
    }
}

#[get("/api/loggers/{name}/config")]
async fn get_logger_config(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let name = path.into_inner();
    let config = LoggerConfig {
        enabled: true,
        interval: 5000,
    };
    Ok(HttpResponse::Ok().json(config))
}

#[put("/api/loggers/{name}/config")]
async fn update_logger_config(
    path: web::Path<String>,
    config: web::Json<LoggerConfig>,
) -> Result<HttpResponse, Error> {
    let name = path.into_inner();
    Ok(HttpResponse::Ok().json(config.into_inner()))
}

#[get("/api/loggers/{name}/status")]
async fn get_logger_status(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let name = path.into_inner();
    let status = LoggerStatus {
        enabled: true,
        running: false,
    };
    Ok(HttpResponse::Ok().json(status))
}

#[post("/api/loggers/{name}/start")]
async fn start_logger(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let name = path.into_inner();
    Ok(HttpResponse::Ok().json(json!({
        "status": "started",
        "logger": name
    })))
}

#[post("/api/loggers/{name}/stop")]
async fn stop_logger(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let name = path.into_inner();
    Ok(HttpResponse::Ok().json(json!({
        "status": "stopped",
        "logger": name
    })))
}

#[get("/api/loggers/{name}/data")]
async fn get_logger_data(
    name: web::Path<String>,
    query: web::Query<QueryParams>,
    data: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let config = data.config.lock().unwrap();
    
    let mut conditions = Vec::new();
    
    if let Some(start_time) = query.start_time {
        conditions.push(format!("timestamp >= {}", start_time));
    }
    
    if let Some(end_time) = query.end_time {
        conditions.push(format!("timestamp <= {}", end_time));
    }
    
    if let Some(ref filter) = query.filter {
        conditions.push(format!("({filter})"));
    }
    
    let query_str = if conditions.is_empty() {
        "1=1".to_string()
    } else {
        conditions.join(" AND ")
    };
    
    let db_path = match name.as_str() {
        "screen" => PathBuf::from(&config.screen.output_dir).join("screen.db"),
        "camera" => PathBuf::from(&config.camera.output_dir).join("camera.db"),
        "microphone" => PathBuf::from(&config.microphone.output_dir).join("microphone.db"),
        "processes" => PathBuf::from(&config.processes.output_dir).join("processes.db"),
        _ => return Err(ApiError::NotFound("Logger type not found".to_string())),
    };
    
    let conn = match Connection::open(&db_path) {
        Ok(conn) => conn,
        Err(e) => {
            return Err(ApiError::DatabaseError(format!("Failed to open database: {}", e)));
        }
    };
    
    let table_name = match name.as_str() {
        "screen" => "screenshots",
        "camera" => "frames",
        "microphone" => "recordings",
        "processes" => "processes",
        _ => return Err(ApiError::NotFound("Logger type not found".to_string())),
    };
    
    match execute_query_on_table(&conn, table_name, &query_str) {
        Ok(results) => {
            let limited_results = if let Some(limit) = query.limit {
                results.into_iter().take(limit as usize).collect::<Vec<_>>()
            } else {
                results
            };
            
            Ok(HttpResponse::Ok().json(limited_results))
        },
        Err(e) => {
            Err(ApiError::DatabaseError(format!("Query execution failed: {}", e)))
        }
    }
}

fn save_config(config: &config_crate::Config) -> std::io::Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "Could not determine home directory")
    })?;
    
    #[cfg(feature = "dev")]
    let config_path: PathBuf = "dev-config.toml".into();
    
    #[cfg(not(feature = "dev"))]
    let config_path: PathBuf = [home_dir.to_str().unwrap(), ".config/lifelog/config.toml"]
        .iter()
        .collect();
    
    let config_str = toml::to_string(config).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to serialize config: {}", e))
    })?;
    
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    std::fs::write(config_path, config_str)
}

#[post("/api/auth/login")]
async fn login(
    auth_state: web::Data<AuthState>,
    login_req: web::Json<lifelog_server::auth::LoginRequest>,
) -> Result<HttpResponse, ApiError> {
    auth_handlers::login(auth_state, login_req).await
}

#[get("/api/auth/profile")]
async fn get_profile(
    auth_state: web::Data<AuthState>,
    claims: web::ReqData<Claims>,
) -> Result<HttpResponse, ApiError> {
    auth_handlers::get_profile(auth_state, claims).await
}

#[get("/api/settings")]
async fn get_settings(_claims: web::ReqData<Claims>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(json!({
        "theme": "dark",
        "autoRefresh": true,
        "refreshInterval": 30,
        "logLevel": "info"
    })))
}

#[put("/api/settings")]
async fn update_settings(
    _claims: web::ReqData<Claims>,
    _settings: web::Json<serde_json::Value>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Settings updated successfully"
    })))
}

#[get("/api/loggers")]
async fn get_loggers(
    data: web::Data<AppState>,
) -> Result<HttpResponse, ApiError> {
    let loggers = &data.loggers;
    let mut logger_list = Vec::new();
    
    for (_, logger) in loggers {
        let logger = logger.lock().unwrap();
        logger_list.push(logger.clone());
    }
    
    Ok(HttpResponse::Ok().json(ApiResponse {
        status: "success".to_string(),
        data: logger_list,
    }))
}

#[post("/api/loggers/screen/capture")]
async fn capture_screen(_data: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    // TODO: Implement screen capture
    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": {
            "message": "Screen capture not implemented yet"
        }
    })))
}

#[post("/api/loggers/microphone/record/start")]
async fn start_recording(_data: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    // TODO: Implement start recording
    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": {
            "message": "Start recording not implemented yet"
        }
    })))
}

#[post("/api/loggers/microphone/record/stop")]
async fn stop_recording(_data: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    // TODO: Implement stop recording
    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": {
            "message": "Stop recording not implemented yet"
        }
    })))
}

#[post("/api/loggers/microphone/record/pause")]
async fn pause_recording(_data: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    // TODO: Implement pause recording
    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": {
            "message": "Pause recording not implemented yet"
        }
    })))
}

#[post("/api/loggers/microphone/record/resume")]
async fn resume_recording(_data: web::Data<AppState>) -> Result<HttpResponse, ApiError> {
    // TODO: Implement resume recording
    Ok(HttpResponse::Ok().json(json!({
        "status": "success",
        "data": {
            "message": "Resume recording not implemented yet"
        }
    })))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();
    
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Initialize SurrealDB
    match db::init_db().await {
        Ok(_) => tracing::info!("SurrealDB initialized successfully"),
        Err(e) => {
            tracing::error!("Failed to initialize SurrealDB: {}", e);
            std::process::exit(1);
        }
    }
    
    // Create database schemas
    match LoggerDb::create_schemas().await {
        Ok(_) => tracing::info!("SurrealDB schemas created successfully"),
        Err(e) => tracing::warn!("Failed to create SurrealDB schemas: {}", e),
    }
    
    let server_ip = env::var("SERVER_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    
    tracing::info!("Starting server at {}:{}", server_ip, server_port);
    
    HttpServer::new(move || {
        let cors = configure_cors();
        let auth = HttpAuthentication::bearer(JwtAuth::validator);
        
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .service(auth_handlers::login)
                            .service(auth_handlers::get_profile)
                    )
                    .service(
                        web::scope("/loggers")
                            .wrap(auth)
                            .service(get_logger_config)
                            .service(update_logger_config)
                            .service(get_logger_status)
                            .service(start_logger)
                            .service(stop_logger)
                            .route("/{name}/data", web::get().to(handlers::get_logger_data))
                            .route("/{name}/count", web::get().to(handlers::get_logger_count))
                            .route("/camera/capture", web::post().to(handlers::insert_camera_frame))
                            .route("/screen/capture", web::post().to(handlers::insert_screen_capture))
                            .route("/microphone/record", web::post().to(handlers::insert_microphone_recording))
                            .route("/processes/data", web::post().to(handlers::insert_process_data))
                    )
            )
            .service(health_check)
    })
    .bind(format!("{}:{}", server_ip, server_port))?
    .run()
    .await
}
