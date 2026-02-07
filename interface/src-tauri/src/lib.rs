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
        load_config, CameraConfig, ConfigManager, MicrophoneConfig, ProcessesConfig, ScreenConfig,
        TextUploadConfig,
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
