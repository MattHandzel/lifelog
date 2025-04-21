use serde::{Deserialize, Serialize};

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
