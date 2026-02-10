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
    CollectorConfig, GeoConfig, HyprlandConfig, InputLoggerConfig, MicrophoneConfig, MouseConfig,
    NetworkConfig, ProcessesConfig, ScreenConfig, ShellHistoryConfig, SystemConfig,
    SystemPerformanceConfig, TextUploadConfig, WeatherConfig, WifiConfig,
};

pub fn load_config() -> CollectorConfig {
    let home_dir = directories::BaseDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    let _lifelog_home_dir = env::var("LIFELOG_HOME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir.clone());

    #[cfg(feature = "dev")]
    let config_path: PathBuf = "dev-config.toml".into();

    #[cfg(not(feature = "dev"))]
    let config_path: PathBuf = home_dir.join(".config/lifelog/config.toml");

    tracing::info!(path = ?config_path, "Loading config file");

    let config_str = if config_path.exists() {
        fs::read_to_string(&config_path).unwrap_or_else(|_| String::new())
    } else {
        tracing::warn!(path = ?config_path, "Config file not found, creating default");
        let default_config = create_default_config();
        let config_str = toml::to_string(&default_config).unwrap_or_default();
        if let Some(parent) = config_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&config_path, &config_str);
        config_str
    };

    if config_str.is_empty() {
        return create_default_config();
    }

    // The generated proto config structs do not guarantee `#[serde(default)]`, so missing
    // fields can fail deserialization after we add new config options. To keep config files
    // forwards/backwards compatible, we merge the user config on top of defaults at the TOML
    // value level before deserializing into `CollectorConfig`.
    let default_config = create_default_config();
    let default_toml: toml::Value =
        toml::from_str(&toml::to_string(&default_config).unwrap_or_else(|_| String::new()))
            .unwrap_or(toml::Value::Table(Default::default()));

    let user_toml: toml::Value = match toml::from_str(&replace_home_dir_in_path(config_str)) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!(error = %e, "Failed to parse config file TOML, using defaults");
            return default_config;
        }
    };

    let mut merged = default_toml;
    merge_toml(&mut merged, user_toml);

    match toml::from_str::<CollectorConfig>(&toml::to_string(&merged).unwrap_or_default()) {
        Ok(config) => config,
        Err(e) => {
            tracing::error!(error = %e, "Failed to deserialize merged config, using defaults");
            default_config
        }
    }
}

fn merge_toml(base: &mut toml::Value, overlay: toml::Value) {
    match (base, overlay) {
        (toml::Value::Table(base_tbl), toml::Value::Table(overlay_tbl)) => {
            for (k, v) in overlay_tbl {
                match base_tbl.get_mut(&k) {
                    Some(existing) => merge_toml(existing, v),
                    None => {
                        base_tbl.insert(k, v);
                    }
                }
            }
        }
        (base_slot, overlay_val) => {
            *base_slot = overlay_val;
        }
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
        self.config.camera.clone().unwrap_or_default()
    }

    pub fn set_camera_config(&mut self, camera_config: CameraConfig) {
        self.config.camera = Some(camera_config);
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let home_dir = directories::BaseDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/tmp"));

        #[cfg(feature = "dev")]
        let config_path: PathBuf = "dev-config.toml".into();

        #[cfg(not(feature = "dev"))]
        let config_path: PathBuf = home_dir.join(".config/lifelog/config.toml");

        let config_str = toml::to_string(&self.config)
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
