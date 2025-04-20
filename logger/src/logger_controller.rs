use tokio::task::{AbortHandle, JoinHandle};
use crate::modules::*;
use std::sync::Arc;
use config;

pub struct LoggerController {
    task: Option<AbortHandle>,
    config: Arc<config::Config>,
}

impl LoggerController {
    pub fn new(config: Arc<config::Config>) -> Self {
        Self {
            task: None,
            config,
        }
    }

    pub fn handshake(&mut self) {

    }

    pub fn listen(&mut self) {

    }

    pub fn start(&mut self, ) {
        let config = Arc::clone(&self.config);

        self.handshake(); // handle error!

        let mut handles = Vec::new();

        if config.screen.enabled {
            let config_clone = Arc::clone(&self.config);
            handles.push(screen::start_logger(&config_clone.screen));
        }

        // let handle = tokio::spawn(async move {
        //     match name.as_str() {
        //         "screen" => screen::start_logger(&config.screen).await,
        //         "microphone" => microphone::start_logger(&config.microphone).await,
        //         // ... other loggers
        //         _ => panic!("Unknown logger"),
        //     }
        // })
        // .abort_handle();
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
