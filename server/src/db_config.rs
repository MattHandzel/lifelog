// Database configuration and environment setup
use serde::{Deserialize, Serialize};
use std::env;

/// SurrealDB configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurrealDbConfig {
    /// Connection URL (e.g., "ws://localhost:8000")
    pub url: String,
    
    /// Username for authentication
    pub username: String,
    
    /// Password for authentication
    pub password: String,
    
    /// Namespace to use
    pub namespace: String,
    
    /// Database to use
    pub database: String,
}

impl Default for SurrealDbConfig {
    fn default() -> Self {
        Self {
            url: "ws://localhost:8000".to_string(),
            username: "root".to_string(),
            password: "root".to_string(),
            namespace: "lifelog".to_string(),
            database: "data".to_string(),
        }
    }
}

impl SurrealDbConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            url: env::var("SURREALDB_URL").unwrap_or_else(|_| "ws://localhost:8000".to_string()),
            username: env::var("SURREALDB_USER").unwrap_or_else(|_| "root".to_string()),
            password: env::var("SURREALDB_PASS").unwrap_or_else(|_| "root".to_string()),
            namespace: env::var("SURREALDB_NS").unwrap_or_else(|_| "lifelog".to_string()),
            database: env::var("SURREALDB_DB").unwrap_or_else(|_| "data".to_string()),
        }
    }
}

/// Add SurrealDB settings to .env file template
pub fn surreal_db_env_template() -> String {
    r#"
# SurrealDB Configuration
SURREALDB_URL=ws://localhost:8000
SURREALDB_USER=root
SURREALDB_PASS=root
SURREALDB_NS=lifelog
SURREALDB_DB=data
"#.to_string()
} 