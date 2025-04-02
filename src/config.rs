use crate::utils::replace_home_dir_in_path;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// TODO: Implement default for all configs
// TODO: Make it so that there is a default directory
// TODO: How do other projects do configs

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub screen: ScreenConfig,
    pub camera: CameraConfig,
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
    pub hyprland: HyprlandConfig,
    pub server: ServerConfig,
    pub input_logger: InputLoggerConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_ip")]
    pub ip: String,

    #[serde(default = "default_server_port")]
    pub port: u16,

    pub folder_dir: PathBuf,
}

// TODO: Should this be changed fromo https?
fn default_server_ip() -> String {
    "https://localhost".to_string()
}

fn default_server_port() -> u16 {
    7182 // randomish number
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,

    #[serde(default = "default_network_interval")]
    pub interval: f64,

    pub output_dir: PathBuf,
}

fn default_network_interval() -> f64 {
    60.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessesConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_processes_interval")]
    pub interval: f64,

    pub output_dir: PathBuf,
}

fn default_processes_interval() -> f64 {
    60.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MicrophoneConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub output_dir: PathBuf,

    #[serde(default = "default_microphone_sample_rate")]
    pub sample_rate: u32,

    #[serde(default = "default_microphone_chunk_duration_secs")]
    pub chunk_duration_secs: u64,

    #[serde(default = "default_timestamp_format")]
    pub timestamp_format: String,

    #[serde(default = "default_microphone_bits_per_sample")]
    pub bits_per_sample: u32,

    #[serde(default = "default_microphone_channels")]
    pub channels: u32,
}

fn default_microphone_sample_rate() -> u32 {
    44100
}
fn default_microphone_chunk_duration_secs() -> u64 {
    300
}
fn default_microphone_bits_per_sample() -> u32 {
    16
}
fn default_microphone_channels() -> u32 {
    2
}

// Add new config structs
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemPerformanceConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_system_performance_interval")]
    pub interval: f64,
    pub output_dir: PathBuf,

    #[serde(default = "default_true")]
    pub log_cpu: bool,
    #[serde(default = "default_true")]
    pub log_memory: bool,
    #[serde(default = "default_true")]
    pub log_disk: bool,
}

fn default_system_performance_interval() -> f64 {
    10.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AmbientConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,

    #[serde(default = "default_ambient_interval")]
    pub interval: f64,

    pub output_dir: PathBuf,

    pub temperature_sensor_path: Option<String>,
    pub humidity_sensor_path: Option<String>,
}

fn default_ambient_interval() -> f64 {
    60.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,

    #[serde(default = "default_weather_interval")]
    pub interval: f64,

    pub output_dir: PathBuf,

    pub api_key: String,

    pub latitude: f64,
    pub longitude: f64,
}

fn default_weather_interval() -> f64 {
    60.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub output_dir: PathBuf,

    #[serde(default = "default_audio_sample_rate")]
    pub sample_rate: u32,
    #[serde(default = "default_audio_chunk_duration_secs")]
    pub chunk_duration_secs: u64,
}

// TODO: Should these default values be changed/decreased to match the storage capacity of a
// general user?
fn default_audio_sample_rate() -> u32 {
    44100
}

fn default_audio_chunk_duration_secs() -> u64 {
    300
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_geo_interval")]
    pub interval: f64,

    pub output_dir: PathBuf,
    #[serde(default = "default_geo_ip_fallback")]
    pub use_ip_fallback: bool,
}

fn default_geo_interval() -> f64 {
    60.0
}
fn default_geo_ip_fallback() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WifiConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_wifi_interval")]
    pub interval: f64,
    pub output_dir: PathBuf,
    #[serde(default = "default_scan_command")]
    pub scan_command: String,
}

fn default_wifi_interval() -> f64 {
    300.0 // 5 minutes
}

// TODO: Make this a function that returns the default scan command based on the OS
fn default_scan_command() -> String {
    "nmcli device wifi list".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyboardConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MouseConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScreenConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_screen_interval")]
    pub interval: f64,
    pub output_dir: PathBuf,
    #[serde(default = "default_screen_program")]
    pub program: String,
    #[serde(default = "default_timestamp_format")]
    pub timestamp_format: String,
}

fn default_screen_interval() -> f64 {
    60.0
}

// TODO: Make this a function that returns the default screen program based on the OS
fn default_screen_program() -> String {
    "gnome-screenshot".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HyprlandConfig {
    // TODO: Make this be based on OS and DE
    #[serde(default = "default_false")]
    pub enabled: bool,

    #[serde(default = "default_hyprland_interval")]
    pub interval: f64,
    pub output_dir: PathBuf,

    #[serde(default = "default_true")]
    pub log_clients: bool,
    #[serde(default = "default_true")]
    pub log_activewindow: bool,
    #[serde(default = "default_true")]
    pub log_workspace: bool,
    #[serde(default = "default_true")]
    pub log_active_monitor: bool,
    #[serde(default = "default_true")]
    pub log_devices: bool,
}

fn default_hyprland_interval() -> f64 {
    1.0
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CameraConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_camera_interval")]
    pub interval: f64,
    pub output_dir: PathBuf,
    pub device: String,
    pub resolution: (u32, u32),
    pub fps: u32,
    pub timestamp_format: String,
}

fn default_camera_interval() -> f64 {
    10.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputLoggerConfig {
    /// Path to store the database file
    pub output_dir: PathBuf,
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Log keyboard events
    #[serde(default = "default_true")]
    pub log_keyboard: bool,

    /// Log mouse button events
    #[serde(default = "default_true")]
    pub log_mouse_buttons: bool,

    /// Log mouse movement
    #[serde(default = "default_true")]
    pub log_mouse_movement: bool,

    /// Log mouse wheel events
    #[serde(default = "default_true")]
    pub log_mouse_wheel: bool,

    /// Log device connection events
    #[serde(default = "default_true")]
    pub log_devices: bool,

    #[serde(default = "default_mouse_interval")]
    pub mouse_interval: f64,
}

fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_mouse_interval() -> f64 {
    1.0
}

fn default_timestamp_format() -> String {
    "%Y-%m-%d_%H-%M-%S.%3f%Z".to_string()
}

//impl Default for InputLoggerConfig {
//    fn default() -> Self {
//        Self {
//            log_keyboard: true,
//            log_mouse_buttons: true,
//            log_mouse_movement: true,
//            log_mouse_wheel: true,
//            log_devices: true,
//            mouse_interval: 1,
//        }
//    }
//}

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

    toml::from_str(replace_home_dir_in_path(config_str).as_str())
        .expect("Failed to parse config.toml")
}
