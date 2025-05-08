// Export all modules needed by the main binary
// pub mod config; - Using common config crate instead
pub mod embed {
    // Empty placeholder module
}

// No longer needed with the new API-based architecture
// pub mod modules {
//     // Empty placeholder module
// }

pub mod prelude;
pub mod setup;
pub mod storage;
// pub mod utils; - Using common utils crate instead

// Re-export commonly used items
pub use config::*;
// pub use modules::*;  // No longer using direct module access
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
}

// New API client module for communicating with the server
pub mod api_client {
    use reqwest::Client;
    use serde::{Deserialize, Serialize};
    use std::env;
    use std::time::Duration;

    // API base URL - defaults to localhost:8080 if not set
    pub fn get_api_base_url() -> String {
        env::var("VITE_API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
    }

    // Create a new API client with reasonable defaults
    pub fn create_client() -> Client {
        Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client")
    }

    // Generic API response structure
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ApiResponse<T> {
        pub success: bool,
        pub data: Option<T>,
        pub error: Option<String>,
    }
}
