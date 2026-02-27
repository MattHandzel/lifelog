pub use lifelog_types::ServerConfig;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utils::replace_home_dir_in_path;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TransformSpec {
    pub id: String,
    pub enabled: bool,
    pub source_origin: String,
    pub language: Option<String>,
}

/// Load transform specs from `LIFELOG_TRANSFORMS_JSON`.
///
/// Example:
/// `[{"id":"ocr","enabled":true,"source_origin":"*:screen","language":"eng"}]`
///
/// Fallback default: OCR enabled for all screen origins.
pub fn load_transform_specs() -> Vec<TransformSpec> {
    let fallback = vec![TransformSpec {
        id: "ocr".to_string(),
        enabled: true,
        source_origin: "*:screen".to_string(),
        language: Some("eng".to_string()),
    }];

    if let Ok(v) = std::env::var("LIFELOG_TRANSFORMS_JSON") {
        if !v.trim().is_empty() {
            return match serde_json::from_str::<Vec<TransformSpec>>(&v) {
                Ok(specs) if !specs.is_empty() => specs,
                Ok(_) => fallback,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "Failed to parse LIFELOG_TRANSFORMS_JSON; using fallback"
                    );
                    fallback
                }
            };
        }
    }

    let path = std::env::var("LIFELOG_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| crate::default_lifelog_config_path());
    let cfg_text = match std::fs::read_to_string(&path) {
        Ok(v) => v,
        Err(_) => return fallback,
    };
    let parsed: toml::Value = match toml::from_str(&replace_home_dir_in_path(cfg_text)) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(
                error = %e,
                path = ?path,
                "Failed to parse lifelog-config.toml for transforms; using fallback"
            );
            return fallback;
        }
    };
    let normalized = normalize_toml_keys(parsed);
    let Some(transforms_val) = normalized.get("transforms") else {
        return fallback;
    };
    let raw = toml::to_string(transforms_val).unwrap_or_default();

    match toml::from_str::<Vec<TransformSpec>>(&raw) {
        Ok(specs) if !specs.is_empty() => specs,
        Ok(_) => fallback,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to parse transforms in lifelog-config.toml; using fallback"
            );
            fallback
        }
    }
}

fn normalize_toml_keys(value: toml::Value) -> toml::Value {
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
