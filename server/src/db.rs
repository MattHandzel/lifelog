// Database connection and operations for SurrealDB
use std::sync::Arc;
use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::sync::Mutex;
use surrealdb::engine::local::Db;

use crate::db_config::SurrealDbConfig;

// Global SurrealDB client
static DB: OnceCell<Arc<Mutex<Surreal<Client>>>> = OnceCell::new();

/// Initialize the SurrealDB connection
pub async fn init_db() -> Result<()> {
    let config = SurrealDbConfig::from_env();
    
    tracing::info!("Initializing SurrealDB connection to {}", config.url);
    
    let db = Surreal::new::<Ws>(&config.url).await?;
    
    db.signin(Root {
        username: &config.username,
        password: &config.password,
    }).await?;
    
    db.use_ns(&config.namespace).use_db(&config.database).await?;
    
    DB.set(Arc::new(Mutex::new(db))).unwrap();
    
    tracing::info!("SurrealDB connection initialized successfully");
    Ok(())
}

/// Get a reference to the global DB client
pub fn get_db() -> Arc<Mutex<Surreal<Client>>> {
    DB.get()
        .expect("Database not initialized. Call init_db() first.")
        .clone()
}

/// Logger types
pub enum LoggerType {
    Screen,
    Camera,
    Microphone,
    Processes,
    Hyprland,
}

impl LoggerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::Camera => "camera",
            Self::Microphone => "microphone",
            Self::Processes => "processes",
            Self::Hyprland => "hyprland",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "screen" => Some(Self::Screen),
            "camera" => Some(Self::Camera),
            "microphone" => Some(Self::Microphone),
            "processes" => Some(Self::Processes),
            "hyprland" => Some(Self::Hyprland),
            _ => None,
        }
    }
}

/// Generic database operations for loggers
pub struct LoggerDb;

impl LoggerDb {
    /// Create table schemas for all loggers
    pub async fn create_schemas() -> Result<()> {
        let db = get_db();
        let db = db.lock().await;
        
        // Screen logger schema
        db.query("DEFINE TABLE screen").await?;
        db.query(r#"
            DEFINE FIELD timestamp ON TABLE screen TYPE number;
            DEFINE FIELD path ON TABLE screen TYPE string;
            DEFINE FIELD width ON TABLE screen TYPE number;
            DEFINE FIELD height ON TABLE screen TYPE number;
            DEFINE INDEX screen_timestamp ON TABLE screen COLUMNS timestamp;
        "#).await?;
        
        // Camera logger schema
        db.query("DEFINE TABLE camera").await?;
        db.query(r#"
            DEFINE FIELD timestamp ON TABLE camera TYPE number;
            DEFINE FIELD path ON TABLE camera TYPE string;
            DEFINE FIELD width ON TABLE camera TYPE number;
            DEFINE FIELD height ON TABLE camera TYPE number;
            DEFINE INDEX camera_timestamp ON TABLE camera COLUMNS timestamp;
        "#).await?;
        
        // Microphone logger schema
        db.query("DEFINE TABLE microphone").await?;
        db.query(r#"
            DEFINE FIELD timestamp ON TABLE microphone TYPE number;
            DEFINE FIELD path ON TABLE microphone TYPE string;
            DEFINE FIELD duration ON TABLE microphone TYPE number;
            DEFINE FIELD size ON TABLE microphone TYPE number;
            DEFINE FIELD created_at ON TABLE microphone TYPE string;
            DEFINE INDEX microphone_timestamp ON TABLE microphone COLUMNS timestamp;
        "#).await?;
        
        // Processes logger schema
        db.query("DEFINE TABLE processes").await?;
        db.query(r#"
            DEFINE FIELD timestamp ON TABLE processes TYPE number;
            DEFINE FIELD pid ON TABLE processes TYPE number;
            DEFINE FIELD ppid ON TABLE processes TYPE number;
            DEFINE FIELD name ON TABLE processes TYPE string;
            DEFINE FIELD exe ON TABLE processes TYPE string;
            DEFINE FIELD cmdline ON TABLE processes TYPE string;
            DEFINE FIELD status ON TABLE processes TYPE string;
            DEFINE FIELD cpu_usage ON TABLE processes TYPE number;
            DEFINE FIELD memory_usage ON TABLE processes TYPE number;
            DEFINE FIELD threads ON TABLE processes TYPE number;
            DEFINE FIELD user ON TABLE processes TYPE string;
            DEFINE FIELD start_time ON TABLE processes TYPE number;
            DEFINE INDEX processes_timestamp ON TABLE processes COLUMNS timestamp;
            DEFINE INDEX processes_name ON TABLE processes COLUMNS name;
        "#).await?;
        
        tracing::info!("SurrealDB schemas created successfully");
        Ok(())
    }
    
    /// Get records with pagination and filtering
    pub async fn get_records(
        logger_type: LoggerType,
        start_time: Option<f64>,
        end_time: Option<f64>,
        limit: Option<u32>,
        filter: Option<String>,
        page: Option<usize>,
        page_size: Option<usize>,
    ) -> Result<Vec<serde_json::Value>> {
        let db = get_db();
        let db = db.lock().await;
        let table = logger_type.as_str();
        
        let mut query = format!("SELECT * FROM {}", table);
        let mut conditions = Vec::new();
        let mut params = surrealdb::sql::Object::default();
        
        if let Some(start) = start_time {
            conditions.push("timestamp >= $start_time");
            params.insert("start_time".into(), start.into());
        }
        
        if let Some(end) = end_time {
            conditions.push("timestamp <= $end_time");
            params.insert("end_time".into(), end.into());
        }
        
        if let Some(custom_filter) = filter {
            conditions.push(&custom_filter);
        }
        
        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }
        
        query.push_str(" ORDER BY timestamp DESC");
        
        if let (Some(p), Some(ps)) = (page, page_size) {
            let offset = (p - 1) * ps;
            query.push_str(&format!(" LIMIT {} START {}", ps, offset));
        } else if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }
        
        let result = db.query(&query).bind(params).await?;
        let records: Vec<serde_json::Value> = result.take(0)?;
        
        Ok(records)
    }
    
    /// Insert a new record
    pub async fn insert_record(logger_type: LoggerType, record: serde_json::Value) -> Result<serde_json::Value> {
        let db = get_db();
        let db = db.lock().await;
        let table = logger_type.as_str();
        
        let result = db.create(table).content(record).await?;
        Ok(result)
    }
    
    /// Count total records
    pub async fn count_records(logger_type: LoggerType) -> Result<u64> {
        let db = get_db();
        let db = db.lock().await;
        let table = logger_type.as_str();
        
        let result = db.query(format!("SELECT count() FROM {}", table)).await?;
        let count: Vec<serde_json::Value> = result.take(0)?;
        
        if let Some(count_obj) = count.first() {
            if let Some(count_val) = count_obj.get("count") {
                if let Some(count) = count_val.as_u64() {
                    return Ok(count);
                }
            }
        }
        
        Ok(0)
    }
} 