// Export all modules needed by the main binary
pub mod config;
pub mod embed;
pub mod modules;
pub mod prelude;
pub mod setup;
pub mod utils;

// Re-export commonly used items
pub use config::*;
pub use modules::*;
pub use setup::*;
pub use utils::*;

// Define utility functions for loading configs
pub mod config_utils {
    pub use config::{
        load_config, CameraConfig, Config, ConfigManager, ProcessesConfig, ScreenConfig,
    };

    pub fn load_text_upload_config() -> config::TextUploadConfig {
        let config = load_config();
        config.text_upload.clone()
    }

    pub fn load_processes_config() -> config::ProcessesConfig {
        let config = load_config();
        config.processes.clone()
    }

    pub fn load_screen_config() -> config::ScreenConfig {
        let config = load_config();
        config.screen.clone()
    }

    pub fn load_camera_config() -> config::CameraConfig {
        let config = load_config();
        config.camera.clone()
    }
}
