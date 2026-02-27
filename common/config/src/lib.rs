use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use utils::replace_home_dir_in_path;

mod policy_config;
mod server_config;
pub use policy_config::*;
pub use server_config::*;

// Re-export all config types from lifelog_types
pub use lifelog_types::{
    AmbientConfig, AudioConfig, BrowserHistoryConfig, CameraConfig, ClipboardConfig,
    CollectorConfig, GeoConfig, HyprlandConfig, InputLoggerConfig, KeyboardConfig,
    MicrophoneConfig, MouseConfig, NetworkConfig, ProcessesConfig, ScreenConfig,
    ShellHistoryConfig, SystemConfig, SystemPerformanceConfig, TextUploadConfig, WeatherConfig,
    WifiConfig, WindowActivityConfig,
};

#[cfg(not(feature = "dev"))]
fn home_dir() -> PathBuf {
    directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

pub fn default_lifelog_config_path() -> PathBuf {
    #[cfg(feature = "dev")]
    {
        "lifelog-config.toml".into()
    }

    #[cfg(not(feature = "dev"))]
    {
        home_dir().join(".config/lifelog/lifelog-config.toml")
    }
}

fn load_toml_from_path(path: &PathBuf) -> Option<toml::Value> {
    if !path.exists() {
        return None;
    }
    let raw = fs::read_to_string(path).ok()?;
    let replaced = replace_home_dir_in_path(raw);
    let value: toml::Value = toml::from_str(&replaced).ok()?;
    Some(normalize_toml_keys(value))
}

fn collector_config_from_toml(
    collector_id: &str,
    collector_toml: toml::Value,
) -> Option<CollectorConfig> {
    let mut selected_tbl = collector_toml.as_table()?.clone();
    for required in [
        "host",
        "port",
        "timestampFormat",
        "browser",
        "screen",
        "camera",
        "microphone",
        "processes",
        "hyprland",
    ] {
        if !selected_tbl.contains_key(required) {
            return None;
        }
    }
    selected_tbl
        .entry("id".to_string())
        .or_insert(toml::Value::String(collector_id.to_string()));
    toml::from_str::<CollectorConfig>(&toml::to_string(&toml::Value::Table(selected_tbl)).ok()?)
        .ok()
}

fn collector_from_unified_root(root: &toml::Value) -> Option<CollectorConfig> {
    // Required scalable config form:
    // [collectors.<collector_id>]
    let collectors = root.get("collectors")?.as_table()?;
    if collectors.is_empty() {
        return None;
    }

    let selected_id = env::var("LIFELOG_COLLECTOR_ID").ok().or_else(|| {
        root.get("runtime")
            .and_then(|v| v.get("collectorId"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    })?;

    let selected = collectors.get(&selected_id)?.clone();
    collector_config_from_toml(&selected_id, selected)
}

pub fn load_collector_config_from_unified() -> Option<CollectorConfig> {
    let path = env::var("LIFELOG_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_lifelog_config_path());
    let root = load_toml_from_path(&path)?;
    collector_from_unified_root(&root)
}

pub fn load_server_config_from_unified() -> Option<ServerConfig> {
    let path = env::var("LIFELOG_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_lifelog_config_path());
    let root = load_toml_from_path(&path)?;
    let server = root.get("server")?.clone();
    toml::from_str::<ServerConfig>(&toml::to_string(&server).ok()?).ok()
}

pub fn load_device_aliases_from_unified() -> HashMap<String, String> {
    let path = env::var("LIFELOG_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_lifelog_config_path());
    let Some(root) = load_toml_from_path(&path) else {
        return HashMap::new();
    };
    let Some(value) = root.get("deviceAliases") else {
        return HashMap::new();
    };
    toml::from_str::<HashMap<String, String>>(&toml::to_string(value).unwrap_or_default())
        .unwrap_or_default()
}

pub fn load_collectors_from_unified() -> HashMap<String, CollectorConfig> {
    let path = env::var("LIFELOG_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_lifelog_config_path());
    let Some(root) = load_toml_from_path(&path) else {
        return HashMap::new();
    };
    let Some(collectors) = root.get("collectors").and_then(|v| v.as_table()) else {
        return HashMap::new();
    };

    let mut out = HashMap::new();
    for (collector_id, value) in collectors {
        if let Some(cfg) = collector_config_from_toml(collector_id, value.clone()) {
            out.insert(collector_id.clone(), cfg);
        }
    }
    out
}

pub fn load_config() -> CollectorConfig {
    let cfg = load_collector_config_from_unified().unwrap_or_else(|| {
        panic!(
            "Invalid or missing collector config in {}. No defaults are applied.",
            env::var("LIFELOG_CONFIG_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|_| default_lifelog_config_path())
                .display()
        )
    });
    tracing::info!(
        path = ?env::var("LIFELOG_CONFIG_PATH").ok().map(PathBuf::from).unwrap_or_else(default_lifelog_config_path),
        "Loaded collector config from unified lifelog-config.toml"
    );
    cfg
}

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

fn normalize_toml_keys(value: toml::Value) -> toml::Value {
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

pub fn create_default_config() -> CollectorConfig {
    let home_dir = directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    let lifelog_dir = home_dir.join("lifelog");

    CollectorConfig {
        id: "implement_this".to_string(),
        host: "127.0.0.1".to_string(),
        port: 7190,
        timestamp_format: default_timestamp_format(),
        screen: Some(ScreenConfig {
            enabled: true,
            interval: 20.0,
            output_dir: lifelog_dir.join("screen").display().to_string(),
            program: "gnome-screenshot".to_string(),
            timestamp_format: default_timestamp_format(),
        }),
        browser: Some(BrowserHistoryConfig {
            enabled: true,
            browser_type: "chrome".to_string(),
            input_file: "".to_string(),
            output_file: lifelog_dir.join("browser").display().to_string(),
        }),
        camera: Some(CameraConfig {
            enabled: false,
            interval: 10.0,
            output_dir: home_dir.join("lifelog_camera").display().to_string(),
            device: "/dev/video0".to_string(),
            resolution_x: 640,
            resolution_y: 480,
            fps: 30,
            timestamp_format: default_timestamp_format(),
        }),
        microphone: Some(MicrophoneConfig {
            enabled: true,
            output_dir: lifelog_dir.join("microphone").display().to_string(),
            sample_rate: 44100,
            chunk_duration_secs: 300,
            timestamp_format: default_timestamp_format(),
            bits_per_sample: 16,
            channels: 2,
            capture_interval_secs: 300,
        }),
        processes: Some(ProcessesConfig {
            enabled: true,
            interval: 60.0,
            output_dir: lifelog_dir.join("processes").display().to_string(),
        }),
        hyprland: Some(HyprlandConfig {
            enabled: false,
            interval: 1.0,
            output_dir: home_dir.join("lifelog_hyprland").display().to_string(),
            log_clients: true,
            log_activewindow: true,
            log_workspace: true,
            log_active_monitor: true,
            log_devices: true,
        }),
        weather: Some(WeatherConfig {
            enabled: false,
            interval: 1800.0,
            output_dir: lifelog_dir.join("weather").display().to_string(),
            api_key: "".to_string(),
            latitude: 0.0,
            longitude: 0.0,
        }),
        wifi: Some(WifiConfig {
            enabled: false,
            interval: 300.0,
            output_dir: lifelog_dir.join("wifi").display().to_string(),
            scan_command: "nmcli -t -f SSID,SIGNAL,BSSID device wifi list".to_string(),
        }),
        clipboard: Some(ClipboardConfig {
            enabled: false,
            interval: 2.0,
            output_dir: lifelog_dir.join("clipboard").display().to_string(),
            max_text_bytes: 262_144,
        }),
        shell_history: Some(ShellHistoryConfig {
            enabled: false,
            interval: 2.0,
            output_dir: lifelog_dir.join("shell_history").display().to_string(),
            history_file: home_dir.join(".zsh_history").display().to_string(),
            shell_type: "auto".to_string(),
        }),
        mouse: Some(MouseConfig {
            enabled: false,
            // Minimum capture interval for cursor snapshots. This is intentionally conservative to
            // avoid huge volumes of events.
            interval: 0.25,
            output_dir: lifelog_dir.join("mouse").display().to_string(),
        }),
        window_activity: Some(WindowActivityConfig {
            enabled: false,
            interval: 1.0,
            output_dir: lifelog_dir.join("window_activity").display().to_string(),
            backend: "auto".to_string(),
        }),
        keyboard: Some(KeyboardConfig {
            enabled: false,
            interval: 1.0,
            output_dir: lifelog_dir.join("keystrokes").display().to_string(),
        }),
    }
}

pub struct ConfigManager {
    config: CollectorConfig,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            config: load_config(),
        }
    }

    pub fn get_config(&self) -> &CollectorConfig {
        &self.config
    }

    pub fn get_camera_config(&self) -> CameraConfig {
        self.config
            .camera
            .clone()
            .expect("Missing [collectors.<id>.camera] config")
    }

    pub fn set_camera_config(&mut self, camera_config: CameraConfig) {
        self.config.camera = Some(camera_config);
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let config_path = env::var("LIFELOG_CONFIG_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| default_lifelog_config_path());

        let mut root = load_toml_from_path(&config_path).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Config file missing or invalid at {}. No defaults are applied.",
                    config_path.display()
                ),
            )
        })?;

        let selected_id = env::var("LIFELOG_COLLECTOR_ID")
            .ok()
            .or_else(|| {
                root.get("runtime")
                    .and_then(|v| v.get("collectorId"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| self.config.id.clone());

        let collector_val = toml::from_str::<toml::Value>(
            &toml::to_string(&self.config)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
        )
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if let Some(root_tbl) = root.as_table_mut() {
            let runtime = root_tbl
                .entry("runtime".to_string())
                .or_insert(toml::Value::Table(Default::default()));
            if let Some(runtime_tbl) = runtime.as_table_mut() {
                runtime_tbl.insert(
                    "collectorId".to_string(),
                    toml::Value::String(selected_id.clone()),
                );
            }

            let collectors = root_tbl
                .entry("collectors".to_string())
                .or_insert(toml::Value::Table(Default::default()));
            if let Some(collectors_tbl) = collectors.as_table_mut() {
                collectors_tbl.insert(selected_id, collector_val);
            }
        }

        let config_str = toml::to_string_pretty(&root)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(config_path, config_str)
    }
}

fn default_timestamp_format() -> String {
    "%Y-%m-%d_%H-%M-%S.%3f%Z".to_string()
}
