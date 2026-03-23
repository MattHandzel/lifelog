pub use lifelog_types::ServerConfig;

#[derive(Debug, Clone, Default)]
pub struct NetworkPolicy {
    pub allowed_hosts: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TlsConfig {
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

impl TlsConfig {
    pub fn from_env() -> Self {
        Self {
            cert_path: std::env::var("LIFELOG_TLS_CERT_PATH").ok(),
            key_path: std::env::var("LIFELOG_TLS_KEY_PATH").ok(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.cert_path.is_some() && self.key_path.is_some()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServerDeployConfig {
    pub postgres_url: Option<String>,
    pub tls: TlsConfig,
    pub allow_plaintext: bool,
    pub postgres_max_connections: Option<usize>,
}

pub fn resolve_file_ref(value: &str) -> String {
    if let Some(path) = value.strip_prefix('@') {
        match std::fs::read_to_string(path) {
            Ok(contents) => contents.trim().to_string(),
            Err(e) => {
                tracing::warn!(path = %path, error = %e, "Failed to read file reference");
                value.to_string()
            }
        }
    } else {
        value.to_string()
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
        retention_policy_days: std::collections::HashMap::new(),
        transforms: Vec::new(),
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
    "postgresql://lifelog@127.0.0.1:5432/lifelog".to_string()
}

pub fn default_server_name() -> String {
    "LifelogServer".to_string()
}

pub fn default_database_name() -> String {
    "main".to_string()
}

pub fn default_server_ip() -> String {
    "localhost".to_string()
}

pub fn default_server_port() -> u32 {
    7182
}

pub fn default_server_url() -> String {
    format!("https://{}:{}", default_server_ip(), default_server_port())
}

pub fn normalize_toml_keys(value: toml::Value) -> toml::Value {
    fn snake_to_camel_key(key: &str) -> String {
        let mut out = String::with_capacity(key.len());
        let mut upper = false;
        for ch in key.chars() {
            if ch == '_' {
                upper = true;
                continue;
            }
            if upper {
                out.extend(ch.to_uppercase());
                upper = false;
            } else {
                out.push(ch);
            }
        }
        out
    }

    match value {
        toml::Value::Table(tbl) => {
            let mut out = toml::map::Map::new();
            for (key, val) in tbl {
                let normalized_key = if key.contains('_') {
                    snake_to_camel_key(&key)
                } else {
                    key
                };
                out.insert(normalized_key, normalize_toml_keys(val));
            }
            toml::Value::Table(out)
        }
        toml::Value::Array(items) => {
            toml::Value::Array(items.into_iter().map(normalize_toml_keys).collect())
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_file_ref_plain_value() {
        assert_eq!(resolve_file_ref("hello"), "hello");
    }

    #[test]
    fn resolve_file_ref_missing_file() {
        let result = resolve_file_ref("@/nonexistent/path/12345");
        assert_eq!(result, "@/nonexistent/path/12345");
    }

    #[test]
    fn resolve_file_ref_reads_file() {
        let dir = std::env::temp_dir().join("lifelog_test_file_ref");
        std::fs::write(&dir, "secret_value\n").unwrap();
        let result = resolve_file_ref(&format!("@{}", dir.display()));
        assert_eq!(result, "secret_value");
        std::fs::remove_file(&dir).ok();
    }
}
