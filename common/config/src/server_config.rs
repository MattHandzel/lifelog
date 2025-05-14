use lifelog_macros::lifelog_type;
use serde::{Deserialize, Serialize};

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_ip")]
    pub host: String,

    #[serde(default = "default_server_port")]
    pub port: u32, // TODO: REFACTOR TO u16, because it should be u16 but then I have some trouble
    // for converting from u16 to u32 for the proto buf

    #[serde(default = "default_database_endpoint")]
    pub database_endpoint: String,

    #[serde(default = "default_database_name")]
    pub database_name: String,

    #[serde(default = "default_server_name")]
    pub server_name: String,
}

pub fn default_database_endpoint() -> String {
    "127.0.0.1:7183".to_string()
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_server_ip(),
            port: default_server_port(),
            database_endpoint: default_database_endpoint(),
            database_name: default_database_name(),
            server_name: default_server_name(),
        }
    }
}

pub fn default_server_name() -> String {
    "LifelogServer".to_string()
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

pub fn default_server_port() -> u32{
    50051
}
