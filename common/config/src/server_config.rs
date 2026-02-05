pub use lifelog_proto::ServerConfig;

pub fn default_server_config() -> ServerConfig {
    ServerConfig {
        host: default_server_ip(),
        port: default_server_port(),
        database_endpoint: default_database_endpoint(),
        database_name: default_database_name(),
        server_name: default_server_name(),
        cas_path: default_cas_path(),
    }
}

pub fn default_cas_path() -> String {
    let home_dir = dirs_next::home_dir().expect("Failed to get home directory");
    home_dir
        .join("lifelog")
        .join("cas")
        .to_str()
        .unwrap_or("lifelog/cas")
        .to_string()
}

pub fn default_database_endpoint() -> String {
    "127.0.0.1:7183".to_string()
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

pub fn default_server_port() -> u32 {
    7182
}