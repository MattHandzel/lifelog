use tokio::task::{AbortHandle, JoinHandle};
use crate::modules::*;
use super::logger::{LoggerHandle, DataLogger};
use crate::modules::{
    screen::ScreenLogger,
    microphone::MicrophoneLogger,
    hyprland::HyprlandLogger,
};
use std::sync::Arc;
use config;
use std::collections::HashMap;

pub struct Controller<T> {
    task: Option<AbortHandle>,
    config: Arc<config::Config>,
    handles: HashMap<String, T>,
}

impl Controller<LoggerHandle> {
    pub fn new(config: Arc<config::Config>) -> Self {
        Self {
            task: None,
            config,
            handles: HashMap::new(),
        }
    }

    pub fn handshake(&mut self) {

    }

    pub fn listen(&mut self) {

    }

    pub fn start(&mut self) {
        let config = Arc::clone(&self.config);

        self.handshake(); // handle error!


        if config.screen.enabled {
            let config_clone = Arc::clone(&self.config);
            let logger = ScreenLogger::new(config_clone.screen.clone());
            match logger.expect("REASON").setup() {
                Ok(handle) => {self.handles.insert("screen".to_string(), handle);},
                Err(e)     => eprintln!("screen logger setup error: {:?}", e),
            }
        }

        if config.microphone.enabled {
            let config_clone = Arc::clone(&self.config);
            let logger = MicrophoneLogger::new(config_clone.microphone.clone());
            match logger.expect("REASON").setup() {
                Ok(handle) => {self.handles.insert("microphone".to_string(), handle);},
                Err(e)     => eprintln!("mic logger setup error: {:?}", e),
            }
        }

        if config.hyprland.enabled {
            let config_clone = Arc::clone(&self.config);
            let logger = HyprlandLogger::new(config_clone.hyprland.clone());
            match logger.expect("REASON").setup() {
                Ok(handle) => {self.handles.insert("hyprland".to_string(), handle);},
                Err(e)     => eprintln!("hyprland logger setup error: {:?}", e),
            }
        }
    }

    pub fn send_data(&mut self) {

    }

    pub fn stop(&mut self) {
        if let Some(handle) = &self.task {
            handle.abort();
        }
    }

    pub fn restart(&mut self) {
        self.stop();
        self.start();
    }
}
