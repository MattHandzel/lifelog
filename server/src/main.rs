// TODO: Add support for CRON-like scheduling for:
//       - Data processing
//       - Synching between data sources (cloud data sources, browser history, cliphistory)
// TODO: Add support for queries for the database
// TODO: Add support for natural language to query
//
//
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use surrealdb::engine::local::RocksDb;
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
pub struct ServerConfig {
    pub database_path: PathBuf,
    pub host: String,
    pub port: u16,
    pub database_name: &str,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("~/lifelog/database.db"),
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_name: "main",
        }
    }
}

pub struct Server {
    db: Surreal<RocksDb>,
    host: String,
    port: u16,
}

#[async_trait]
impl Server {
    pub async fn new(config: ServerConfig) -> Result<Self, ServerError> {
        // Initialize database
        let db = Surreal::new::<RocksDb>(config.database_path.clone()).await?;
        db.use_ns("lifelog").use_db(config.database_name).await?;

        Ok(Self {
            db,
            host: config.host,
            port: config.port,
        })
    }
}

// Simplified data structure for example
#[derive(Debug, Serialize, Deserialize)]
pub struct CollectorData {
    collector_id: String,
    raw_data: serde_json::Value,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();
    let server = Server::new(config).await?;

    println!("Server running on {}:{}", server.host, server.port);

    // In real implementation, start server listeners here
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    async fn create_test_server() -> Server {
        let dir = tempdir().unwrap();
        let config = ServerConfig {
            database_path: dir.path().join("test.db"),
            ..Default::default()
        };
        Server::new(config).await.unwrap()
    }
}
