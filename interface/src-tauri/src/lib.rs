pub mod embed {
    // Empty placeholder module
}

pub mod prelude;
pub mod setup;
pub mod storage;
pub use config::*;
pub use setup::*;
pub use utils::*;

// Define utility functions for loading configs
pub mod config_utils {
    pub use config::{
        load_config, CameraConfig, ConfigManager, ProcessesConfig, ScreenConfig,
        TextUploadConfig,
        MicrophoneConfig
    };

    pub fn load_text_upload_config() -> TextUploadConfig {
        TextUploadConfig {
            enabled: false,
            output_dir: "".to_string().into(),
            max_file_size_mb: 0,
            supported_formats: vec![],
        }
    }

    pub fn load_processes_config() -> ProcessesConfig {
        let config = load_config();
        config.processes.clone()
    }

    pub fn load_screen_config() -> ScreenConfig {
        let config = load_config();
        config.screen.clone()
    }

    pub fn load_camera_config() -> CameraConfig {
        let config = load_config();
        config.camera.clone()
    }
    
    pub fn load_microphone_config() -> config::MicrophoneConfig {
        let config = load_config();
        config.microphone.clone()
    }

    // Add saving functions
    pub fn save_screen_config(screen_config: &config::ScreenConfig) {
        match dirs::home_dir() {
            Some(home_dir) => {
                let config_dir = home_dir.join(".lifelog");
                std::fs::create_dir_all(&config_dir).unwrap_or_else(|_| {
                    println!("Could not create config directory");
                });
                let config_file = config_dir.join("screen_config.json");
                let config_json = serde_json::to_string_pretty(screen_config).unwrap_or_else(|_| {
                    println!("Could not serialize screen config");
                    "{}".to_string()
                });
                std::fs::write(config_file, config_json).unwrap_or_else(|_| {
                    println!("Could not write screen config file");
                });
            }
            None => {
                println!("Could not get home directory");
            }
        }
    }

    pub fn save_microphone_config(microphone_config: &config::MicrophoneConfig) {
        match dirs::home_dir() {
            Some(home_dir) => {
                let config_dir = home_dir.join(".lifelog");
                std::fs::create_dir_all(&config_dir).unwrap_or_else(|_| {
                    println!("Could not create config directory");
                });
                let config_file = config_dir.join("microphone_config.json");
                let config_json = serde_json::to_string_pretty(microphone_config).unwrap_or_else(|_| {
                    println!("Could not serialize microphone config");
                    "{}".to_string()
                });
                std::fs::write(config_file, config_json).unwrap_or_else(|_| {
                    println!("Could not write microphone config file");
                });
            }
            None => {
                println!("Could not get home directory");
            }
        }
    }

    pub fn save_text_upload_config(text_config: &config::TextUploadConfig) {
        match dirs::home_dir() {
            Some(home_dir) => {
                let config_dir = home_dir.join(".lifelog");
                std::fs::create_dir_all(&config_dir).unwrap_or_else(|_| {
                    println!("Could not create config directory");
                });
                let config_file = config_dir.join("text_upload_config.json");
                let config_json = serde_json::to_string_pretty(text_config).unwrap_or_else(|_| {
                    println!("Could not serialize text upload config");
                    "{}".to_string()
                });
                std::fs::write(config_file, config_json).unwrap_or_else(|_| {
                    println!("Could not write text upload config file");
                });
            }
            None => {
                println!("Could not get home directory");
            }
        }
    }

    pub fn save_processes_config(processes_config: &config::ProcessesConfig) {
        match dirs::home_dir() {
            Some(home_dir) => {
                let config_dir = home_dir.join(".lifelog");
                std::fs::create_dir_all(&config_dir).unwrap_or_else(|_| {
                    println!("Could not create config directory");
                });
                let config_file = config_dir.join("processes_config.json");
                let config_json = serde_json::to_string_pretty(processes_config).unwrap_or_else(|_| {
                    println!("Could not serialize processes config");
                    "{}".to_string()
                });
                std::fs::write(config_file, config_json).unwrap_or_else(|_| {
                    println!("Could not write processes config file");
                });
            }
            None => {
                println!("Could not get home directory");
            }
        }
    }
}

pub mod api_client {
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::time::Duration;

    pub fn get_api_base_url() -> String {
        env::var("VITE_API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
    }

    pub fn create_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ApiResponse<T> {
        pub success: bool,
        pub data: Option<T>,
        pub error: Option<String>,
    }
}
