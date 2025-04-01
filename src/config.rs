use crate::utils::replace_home_dir_in_path;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub lifelog_dir: PathBuf,
    pub screen: ScreenConfig,
    pub microphone: MicrophoneConfig,
    pub timestamp_format: String,
    pub network: NetworkConfig,
    pub processes: ProcessesConfig,
    pub system_performance: SystemPerformanceConfig,
    pub ambient: AmbientConfig,
    pub weather: WeatherConfig,
    pub audio: AudioConfig,
    pub geolocation: GeoConfig,
    pub wifi: WifiConfig,
    pub text_upload: TextUploadConfig,
}

#[derive(Debug, Deserialize)]
pub struct NetworkConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct ProcessesConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct MicrophoneConfig {
    pub enabled: bool,
    pub output_dir: PathBuf,
    pub sample_rate: u32,
    pub chunk_duration_secs: u64,
    pub timestamp_format: String,
    pub bits_per_sample: u32,
    pub channels: u32,
}

#[derive(Debug, Deserialize)]
pub struct SystemPerformanceConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
    pub log_cpu: bool,
    pub log_memory: bool,
    pub log_disk: bool,
}

#[derive(Debug, Deserialize)]
pub struct AmbientConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
    pub temperature_sensor_path: Option<String>,
    pub humidity_sensor_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WeatherConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
    pub api_key: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Deserialize)]
pub struct AudioConfig {
    pub enabled: bool,
    pub output_dir: PathBuf,
    pub sample_rate: u32,
    pub chunk_duration_secs: u64,
}

#[derive(Debug, Deserialize)]
pub struct GeoConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
    pub use_ip_fallback: bool,
}

#[derive(Debug, Deserialize)]
pub struct WifiConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
    pub scan_command: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScreenConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
    pub program: String,
    pub timestamp_format: String,
}

#[derive(Debug, Deserialize)]
pub struct TextUploadConfig {
    pub enabled: bool,
    pub output_dir: PathBuf,
    pub supported_formats: Vec<String>,
    pub max_file_size_mb: u32,
}

pub fn load_config() -> Config {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    #[cfg(feature = "dev")]
    let config_path: PathBuf = "dev-config.toml".into();

    #[cfg(not(feature = "dev"))]
    let config_path: PathBuf = [home_dir.to_str().unwrap(), ".config/lifelog/config.toml"]
        .iter()
        .collect();

    println!("Using the config file at: {:?}", config_path);
    let config_str = fs::read_to_string(config_path).expect("Failed to read config.toml");
    let config_str = replace_home_dir_in_path(config_str);

    let mut config: Config = toml::from_str(&config_str).expect("Failed to parse config.toml");

    // Ensure the base lifelog directory exists
    fs::create_dir_all(&config.lifelog_dir).expect("Failed to create lifelog base directory");

    config
}
