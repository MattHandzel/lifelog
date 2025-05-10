use lifelog_macros::lifelog_type;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use utils::replace_home_dir_in_path;

use lifelog_core;
mod policy_config;
mod server_config;
use derive_more::From;
pub use policy_config::*;
pub use server_config::*;

// TODO: Implement default for all configs
// TODO: Make it so that there is a default directory
// TODO: How do other projects do configs

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub struct CollectorConfig {
    pub id: String, // This is a human-readable ID for the collector, like "laptop" or "phone"
    pub host: String,
    pub port: u32, //TODO: Refactor code base so this can be u16 instead of u32, this will be the
    //port gRPC server will run on
    pub timestamp_format: String,
    pub browser: BrowserHistoryConfig,
    pub screen: ScreenConfig,
    pub camera: CameraConfig,
    pub microphone: MicrophoneConfig,
    //pub network: NetworkConfig,
    pub processes: ProcessesConfig,
    //pub system_performance: SystemPerformanceConfig,
    //pub ambient: AmbientConfig,
    //pub weather: WeatherConfig,
    //pub audio: AudioConfig,
    //pub geolocation: GeoConfig,
    //pub wifi: WifiConfig,
    pub hyprland: HyprlandConfig,
    //pub server: ServerConfig,
    //pub input_logger: InputLoggerConfig,
    //pub text_upload: TextUploadConfig,
}

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

    #[serde(default = "default_microphone_capture_interval_secs")]
    pub capture_interval_secs: u64,
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
#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

// Weather specific function to load API key from environment
fn load_weather_api_key() -> String {
    env::var("WEATHER_API_KEY").unwrap_or_else(|_| "".to_string())
}

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub struct KeyboardConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub struct MouseConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub struct CameraConfig {
    #[serde(default = "default_false")]
    pub enabled: bool,
    #[serde(default = "default_camera_interval")]
    pub interval: f64,
    #[serde(default = "default_camera_output_dir")]
    pub output_dir: PathBuf,
    #[serde(default = "default_camera_device")]
    pub device: String,

    pub resolution_x: u32,
    pub resolution_y: u32,
    #[serde(default = "default_camera_fps")]
    pub fps: u32,
    #[serde(default = "default_timestamp_format")]
    pub timestamp_format: String,
}

//#[lifelog_type(None)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

fn default_camera_interval() -> f64 {
    10.0
}

fn default_camera_output_dir() -> PathBuf {
    let home_dir = dirs_next::home_dir().expect("Failed to get home directory");
    home_dir.join("lifelog_camera")
}

fn default_camera_device() -> String {
    "/dev/video0".to_string()
}

fn default_camera_resolution() -> Resolution {
    Resolution {
        width: 640,
        height: 480,
    }
}

fn default_camera_fps() -> u32 {
    30
}

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
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

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub struct TextUploadConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub output_dir: PathBuf,
    pub max_file_size_mb: u32,
    pub supported_formats: Vec<String>,
}

pub fn load_config() -> CollectorConfig {
    let home_dir = dirs_next::home_dir().expect("Failed to get home directory");
    let lifelog_home_dir = env::var("LIFELOG_HOME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir.clone());

    #[cfg(feature = "dev")]
    let config_path: PathBuf = "dev-config.toml".into();

    #[cfg(feature = "dev")]
    {
        if !config_path.exists() {
            // If this doesn't exist, create a new config using default
            let default_config = create_default_config();
            let config_str = toml::to_string(&default_config).unwrap_or_else(|e| {
                println!("Warning: Failed to serialize default config: {}", e);
                "".to_string()
            });
            fs::write(&config_path, config_str).unwrap();
        } else {
            println!("Config file exists");
        }
    }

    #[cfg(not(feature = "dev"))]
    let config_path: PathBuf = [home_dir.to_str().unwrap(), ".config/lifelog/config.toml"]
        .iter()
        .collect();

    println!("Using the config file at: {:?}", config_path);
    if config_path.exists() {
        println!("Config file exists");
    } else {
        panic!("Config file does not exist at {:?}", config_path);
    }

    let config_str = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            println!("Failed to read config file: {}", e);
            println!("Creating a new default config file");
            let default_config = create_default_config();
            // Try to save it
            if let Some(parent) = config_path.parent() {
                if let Err(e) = fs::create_dir_all(parent) {
                    println!("Warning: Failed to create config directory: {}", e);
                }
            }

            let config_str = toml::to_string(&default_config).unwrap_or_else(|e| {
                println!("Warning: Failed to serialize default config: {}", e);
                "".to_string()
            });

            if let Err(e) = fs::write(&config_path, &config_str) {
                println!("Warning: Failed to write default config: {}", e);
            }

            return default_config;
        }
    };

    // Try to parse the config file, but provide defaults if parsing fails
    match toml::from_str::<CollectorConfig>(&replace_home_dir_in_path(config_str)) {
        Ok(config) => config,
        Err(e) => {
            panic!("Failed to parse config file: {}", e);

            //println!(
            //    "Failed to parse config.toml: {}. Using default config instead.",
            //    e
            //);
            //let default_config = create_default_config();
            //
            //// Try to save the default config to help the user
            //let config_str = toml::to_string(&default_config).unwrap_or_else(|e| {
            //    println!("Warning: Failed to serialize default config: {}", e);
            //    "".to_string()
            //});
            //
            //let backup_path = config_path.with_extension("toml.bak");
            //if let Err(e) = fs::copy(&config_path, &backup_path) {
            //    println!("Warning: Failed to create backup of original config: {}", e);
            //} else {
            //    println!("Created backup of original config at {:?}", backup_path);
            //}
            //
            //if let Err(e) = fs::write(&config_path, &config_str) {
            //    println!("Warning: Failed to write fixed config: {}", e);
            //} else {
            //    println!("Wrote fixed config to {:?}", config_path);
            //}
            //
            //default_config
        }
    }
}

// Function to create a default configuration
fn create_default_config() -> CollectorConfig {
    let home_dir = dirs_next::home_dir().expect("Failed to get home directory");
    let lifelog_dir = home_dir.join("lifelog");

    CollectorConfig {
        id: "implement_this".to_string(),
        host: "127.0.0.1".to_string(),
        port: 7190,
        timestamp_format: default_timestamp_format(),
        screen: ScreenConfig {
            enabled: true,
            interval: default_screen_interval(),
            output_dir: lifelog_dir.join("screen"),
            program: default_screen_program(),
            timestamp_format: default_timestamp_format(),
        },
        browser: BrowserHistoryConfig {
            enabled: true,
            browser_type: "chrome".to_string(),
            input_dir: "".to_string().into(), // fixme?
            output_file: lifelog_dir.join("browser"),
        },
        camera: CameraConfig {
            enabled: default_false(),
            interval: default_camera_interval(),
            output_dir: default_camera_output_dir(),
            device: default_camera_device(),
            resolution_x: default_camera_resolution().width,
            resolution_y: default_camera_resolution().height,
            fps: default_camera_fps(),
            timestamp_format: default_timestamp_format(),
        },
        //server: ServerConfig {
        //    host: default_server_ip(),
        //    port: , features = default_server_port(),
        //    database_path: default_database_path(),
        //    database_name: default_database_name(),
        //},
        microphone: MicrophoneConfig {
            enabled: default_true(),
            output_dir: lifelog_dir.join("microphone"),
            sample_rate: default_microphone_sample_rate(),
            chunk_duration_secs: default_microphone_chunk_duration_secs(),
            timestamp_format: default_timestamp_format(),
            bits_per_sample: default_microphone_bits_per_sample(),
            channels: default_microphone_channels(),
            capture_interval_secs: default_microphone_capture_interval_secs(),
        },
        //network: NetworkConfig {
        //    enabled: default_false(),
        //    interval: default_network_interval(),
        //    output_dir: home_dir.join("lifelog_network"),
        //},
        processes: ProcessesConfig {
            enabled: default_true(),
            interval: default_processes_interval(),
            output_dir: lifelog_dir.join("processes"),
        },
        //system_performance: SystemPerformanceConfig {
        //    enabled: default_true(),
        //    interval: default_system_performance_interval(),
        //    output_dir: home_dir.join("lifelog_system"),
        //    log_cpu: default_true(),
        //    log_memory: default_true(),
        //    log_disk: default_true(),
        //},
        //ambient: AmbientConfig {
        //    enabled: default_false(),
        //    interval: default_ambient_interval(),
        //    output_dir: home_dir.join("lifelog_ambient"),
        //    temperature_sensor_path: None,
        //    humidity_sensor_path: None,
        //},
        //weather: WeatherConfig {
        //    enabled: default_false(),
        //    interval: default_weather_interval(),
        //    output_dir: home_dir.join("lifelog_weather"),
        //    api_key: load_weather_api_key(),
        //    latitude: 0.0,
        //    longitude: 0.0,
        //},
        //audio: AudioConfig {
        //    enabled: default_true(),
        //    output_dir: home_dir.join("lifelog_audio"),
        //    sample_rate: default_audio_sample_rate(),
        //    chunk_duration_secs: default_audio_chunk_duration_secs(),
        //},
        //geolocation: GeoConfig {
        //    enabled: default_true(),
        //    interval: default_geo_interval(),
        //    output_dir: home_dir.join("lifelog_geo"),
        //    use_ip_fallback: default_geo_ip_fallback(),
        //},
        //wifi: WifiConfig {
        //    enabled: default_true(),
        //    interval: default_wifi_interval(),
        //    output_dir: home_dir.join("lifelog_wifi"),
        //    scan_command: default_scan_command(),
        //},
        hyprland: HyprlandConfig {
            enabled: default_false(),
            interval: default_hyprland_interval(),
            output_dir: home_dir.join("lifelog_hyprland"),
            log_clients: default_true(),
            log_activewindow: default_true(),
            log_workspace: default_true(),
            log_active_monitor: default_true(),
            log_devices: default_true(),
        },
        //input_logger: InputLoggerConfig {
        //    output_dir: home_dir.join("lifelog_input"),
        //    enabled: default_true(),
        //    log_keyboard: default_true(),
        //    log_mouse_buttons: default_true(),
        //    log_mouse_movement: default_true(),
        //    log_mouse_wheel: default_true(),
        //    log_devices: default_true(),
        //    mouse_interval: default_mouse_interval(),
        //},
        //text_upload: TextUploadConfig {
        //    enabled: default_true(),
        //    output_dir: home_dir.join("lifelog_text"),
        //    max_file_size_mb: 10,
        //    supported_formats: vec![
        //        "txt".to_string(),
        //        "md".to_string(),
        //        "json".to_string(),
        //        "csv".to_string(),
        //    ],
        //},
    }
}

pub struct ConfigManager {
    config: CollectorConfig,
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
        self.config.camera.clone()
    }

    pub fn set_camera_config(&mut self, camera_config: CameraConfig) {
        self.config.camera = camera_config;
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let home_dir = dirs_next::home_dir().expect("Failed to get home directory");

        // Dev feature is used for testing
        #[cfg(feature = "dev")]
        let config_path: PathBuf = "dev-config.toml".into();

        #[cfg(not(feature = "dev"))]
        let config_path: PathBuf = [home_dir.to_str().unwrap(), ".config/lifelog/config.toml"]
            .iter()
            .collect();

        let config_str = toml::to_string(&self.config).expect("Failed to serialize config");

        // Create parent directories if they don't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(config_path, config_str)
    }
}

pub fn default_microphone_capture_interval_secs() -> u64 {
    300 // Default to capturing every 5 minutes (300 seconds)
}

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub server: ServerConfig,
    pub collector: CollectorConfig,
    //pub collectors: BTreeMap<String, CollectorConfig>,
}

// #[lifelog_type(None)]
// #[derive(Debug, Clone, Serialize, Deserialize, From)]
// enum BrowserHistoryType {
//     Chrome,
// Firefox}

#[lifelog_type(Config)]
#[derive(Debug, Clone, Serialize, Deserialize, From)]
pub struct BrowserHistoryConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    pub input_dir: PathBuf,

    pub output_file: PathBuf,

    pub browser_type: String,
}

