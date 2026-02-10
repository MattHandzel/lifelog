pub use lifelog_types::ServerConfig;

/// TLS configuration for the server. Loaded from environment variables.
/// This is Rust-only config (not proto) since cert paths are local deployment details.
#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

impl TlsConfig {
    /// Load TLS config from environment variables.
    /// Returns a config with TLS enabled only if both cert and key paths are set.
    pub fn from_env() -> Self {
        Self {
            cert_path: std::env::var("LIFELOG_TLS_CERT_PATH").ok(),
            key_path: std::env::var("LIFELOG_TLS_KEY_PATH").ok(),
        }
    }

    /// Returns true if both cert and key paths are configured.
    pub fn is_enabled(&self) -> bool {
        self.cert_path.is_some() && self.key_path.is_some()
    }
}

pub fn default_server_config() -> ServerConfig {
    ServerConfig {
        host: default_server_ip(),
        port: default_server_port(),
        database_endpoint: default_database_endpoint(),
        database_name: default_database_name(),
        server_name: default_server_name(),
        cas_path: default_cas_path(),
        default_correlation_window_ms: 30_000,
    }
}

pub fn default_cas_path() -> String {
    let home_dir = directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
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

pub fn default_server_url() -> String {
    format!("http://{}:{}", default_server_ip(), default_server_port())
}
