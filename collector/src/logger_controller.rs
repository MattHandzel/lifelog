use tokio::task::{AbortHandle, JoinHandle};

pub struct LoggerController {
    name: String,
    task: Option<AbortHandle>,
    config: Arc<config::AppConfig>,
}

impl LoggerController {
    pub fn new(name: &str, config: Arc<config::AppConfig>) -> Self {
        Self {
            name: name.to_string(),
            task: None,
            config,
        }
    }

    pub fn start(&mut self) {
        let name = self.name.clone();
        let config = Arc::clone(&self.config);

        let handle = tokio::spawn(async move {
            match name.as_str() {
                "screen" => screen::start_logger(&config.screen).await,
                "microphone" => microphone::start_logger(&config.microphone).await,
                // ... other loggers
                _ => panic!("Unknown logger"),
            }
        })
        .abort_handle();

        self.task = Some(handle);
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
