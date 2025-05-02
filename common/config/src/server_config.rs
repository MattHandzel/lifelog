use lifelog_macros::lifelog_type;
use serde::{Deserialize, Serialize};

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_ip")]
    pub host: String,

    #[serde(default = "default_server_port")]
    pub port: u16,

    #[serde(default = "default_database_path")]
    pub database_path: String,

    #[serde(default = "default_database_name")]
    pub database_name: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_server_ip(),
            port: default_server_port(),
            database_path: default_database_path(),
            database_name: default_database_name(),
        }
    }
}

pub fn default_database_path() -> String {
    "surrealkv://".to_string()
}
pub fn default_database_name() -> String {
    "main".to_string()
}

pub fn default_server_ip() -> String {
    "127.0.0.1".to_string()
}

pub fn default_server_port() -> u16 {
    7182
}
