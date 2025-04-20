use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use surrealdb::engine::any;
use surrealdb::engine::local::Db;
use surrealdb::engine::local::Mem;
use surrealdb::engine::local::SurrealKV;
use surrealdb::Connect;
use surrealdb::Response;
use surrealdb::Surreal;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig<'a> {
    pub database_path: &'a str,
    pub host: String,
    pub port: u16,
    pub database_name: &'a str,
}

impl Default for ServerConfig<'_> {
    fn default() -> Self {
        Self {
            database_path: "surrealkv://",
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_name: "main",
        }
    }
}

pub struct Server {
    db: Surreal<Db>,
    host: String,
    port: u16,
}

impl Server {
    pub async fn new(config: &ServerConfig<'_>) -> Result<Self, ServerError> {
        // Initialize database
        let db: Surreal<Db> = (Surreal::new::<Mem>(()).await?);
        db.use_ns("lifelog").use_db(config.database_name).await?;

        Ok(Self {
            db,
            host: config.host.clone(),
            port: config.port.clone(),
        })
    }
}

// TODO: Add support for CRON-like scheduling for:
//       - Data processing
//       - Synching between data sources (cloud data sources, browser history, cliphistory)
// TODO: Add support for queries for the database
// TODO: Add support for natural language to query
//
//

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();
    let server = Server::new(&config).await?;

    println!("Server running on {}:{}", server.host, server.port);

    // In real implementation, start server listeners here
    Ok(())
}
