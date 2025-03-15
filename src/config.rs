use serde::Deserialize;
use std::path::PathBuf;
use std::fs;
use crate::utils::replace_home_dir_in_path;


#[derive(Debug, Deserialize)]
pub struct Config {
    pub keyboard: KeyboardConfig,
    pub mouse: MouseConfig,
    pub screen: ScreenConfig,
    pub camera: CameraConfig,
    pub microphone: MicrophoneConfig,
    pub timestamp_format: String,
    pub network: NetworkConfig,
    pub process: ProcessConfig,
    pub system_performance: SystemPerformanceConfig,
    pub ambient: AmbientConfig,
    pub weather: WeatherConfig,
    pub audio: AudioConfig,
    pub geolocation: GeoConfig,
    pub wifi: WifiConfig,
}

// Add new config structs
#[derive(Debug, Deserialize)]
pub struct SystemPerformanceConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
    pub track_cpu: bool,
    pub track_memory: bool,
    pub track_disk: bool,
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

#[derive(Debug, Deserialize)]
pub struct KeyboardConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct MouseConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct ScreenConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: String,
    pub program: String,
}

#[derive(Debug, Deserialize)]
pub struct CameraConfig {
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
    pub device: String,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub timestamp_format: String,
}

pub fn load_config() -> Config {

    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let config_path: PathBuf = [home_dir.to_str().unwrap(), ".config/lifelog/config.toml"].iter().collect();
    let config_str = fs::read_to_string(config_path).expect("Failed to read config.toml");

    toml::from_str(replace_home_dir_in_path(config_str).as_str()).expect("Failed to parse config.toml")
}



