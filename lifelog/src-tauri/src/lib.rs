// Export all modules needed by the main binary
pub mod config;
pub mod modules;
pub mod setup;
pub mod utils;
pub mod embed;
pub mod prelude;

// Re-export commonly used items
pub use config::*;
pub use modules::*;
pub use setup::*;
pub use utils::*;

// Define utility functions for loading configs
pub mod config_utils {
    pub use crate::config::{
        load_config, Config, ScreenConfig, ProcessesConfig, 
        CameraConfig, ConfigManager,
    };

    pub fn load_text_upload_config() -> crate::config::TextUploadConfig {
        let config = load_config();
        config.text_upload.clone()
    }

    pub fn load_processes_config() -> crate::config::ProcessesConfig {
        let config = load_config();
        config.processes.clone()
    }

    pub fn load_screen_config() -> crate::config::ScreenConfig {
        let config = load_config();
        config.screen.clone()
    }

    pub fn load_camera_config() -> crate::config::CameraConfig {
        let config = load_config();
        config.camera.clone()
    }
}
