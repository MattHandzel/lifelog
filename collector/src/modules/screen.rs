use crate::logger::*;
use async_trait::async_trait;
use chrono::Local;
use config::ScreenConfig;
use data_modalities::screen::ScreenFrame;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};

use image::GenericImageView;
use image::ImageReader;
use lifelog_core::Utc;
use lifelog_core::Uuid;
use lifelog_proto::to_pb_ts;
use std::io::Cursor;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::data_source::{DataSource, DataSourceError, DataSourceHandle};

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct ScreenDataSource {
    config: ScreenConfig,
    logger: ScreenLogger,
    pub buffer: Arc<Mutex<Vec<ScreenFrame>>>,
}

impl ScreenDataSource {
    pub fn new(config: ScreenConfig) -> Result<Self, DataSourceError> {
        let logger = ScreenLogger::new(config.clone()); // handle error?
        Ok(ScreenDataSource {
            config,
            logger: logger?,
            buffer: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub async fn get_data(&mut self) -> Result<Vec<ScreenFrame>, DataSourceError> {
        let buffer_guard = self.buffer.lock().await;

        Ok(buffer_guard.clone())
    }

    pub async fn clear_buffer(&self) -> Result<(), DataSourceError> {
        let mut buffer_guard = self.buffer.lock().await;

        buffer_guard.clear();
        Ok(())
    }
}

#[async_trait]
impl DataSource for ScreenDataSource {
    type Config = ScreenConfig;

    fn new(config: ScreenConfig) -> Result<Self, DataSourceError> {
        ScreenDataSource::new(config)
    }

    fn start(&self) -> Result<DataSourceHandle, DataSourceError> {
        if RUNNING.load(Ordering::SeqCst) {
            tracing::warn!("ScreenDataSource: Start called but task is already running.");
            return Err(DataSourceError::AlreadyRunning);
        }

        tracing::info!("ScreenDataSource: Starting data source task to store in memory");
        RUNNING.store(true, Ordering::SeqCst);

        let source_clone = self.clone();

        let _join_handle = tokio::spawn(async move {
            let task_result = source_clone.run().await;
            tracing::info!(result = ?task_result, "ScreenDataSource (in-memory) background task finished");
            task_result
        });

        tracing::info!("ScreenDataSource: Data source task (in-memory) started successfully");
        let new_join_handle = tokio::spawn(async { Ok(()) });
        Ok(DataSourceHandle {
            join: new_join_handle,
        })
    }

    async fn stop(&mut self) -> Result<(), DataSourceError> {
        RUNNING.store(false, Ordering::SeqCst);
        // FIXME, actually implmenet stop handles
        Ok(())
    }

    async fn run(&self) -> Result<(), DataSourceError> {
        while RUNNING.load(Ordering::SeqCst) {
            match self.logger.log_data().await {
                Ok(image_data_bytes) => {
                    let ts = Utc::now();

                    let img = ImageReader::new(Cursor::new(&image_data_bytes))
                        .with_guessed_format()
                        .map_err(|e| {
                            LoggerError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                        })?
                        .decode()
                        .map_err(|e| {
                            LoggerError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                        })?;

                    let (width, height) = img.dimensions();

                    let captured = ScreenFrame {
                        uuid: Uuid::new_v4().to_string(), //use v6
                        width,
                        height,
                        image_bytes: image_data_bytes,
                        timestamp: to_pb_ts(ts),
                        mime_type: "image/png".to_string(),
                    };

                    let mut store_guard = self.buffer.lock().await;
                    store_guard.push(captured);

                    tracing::debug!(
                        total_images = store_guard.len(),
                        "ScreenDataSource: Stored screen capture in memory"
                    );
                }
                Err(e) => {
                    tracing::error!(error = %e, "ScreenDataSource: Failed to capture screen data for in-memory store");
                }
            }
            sleep(Duration::from_secs_f64(self.config.interval)).await;
        }
        tracing::info!("ScreenDataSource: In-memory run loop finished");
        Ok(())
    }

    fn is_running(&self) -> bool {
        RUNNING.load(Ordering::SeqCst)
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}

#[derive(Clone, Debug)]
pub struct ScreenLogger {
    config: ScreenConfig,
}

impl ScreenLogger {
    pub fn new(config: ScreenConfig) -> Result<Self, LoggerError> {
        Ok(ScreenLogger { config })
    }

    pub fn setup(&self) -> Result<LoggerHandle, LoggerError> {
        DataLogger::setup(self, self.config.clone())
    }

    async fn capture_screenshot_data(&self) -> Result<Vec<u8>, LoggerError> {
        // let temp_file = NamedTempFile::new_in(env::temp_dir())?.into_temp_path();
        // let temp_file_path_str = temp_file.to_str().ok_or_else(|| LoggerError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Invalid temp file path")))?;

        let now = Local::now();
        let _ts = now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1e9;
        let ts_fmt = now.format(&self.config.timestamp_format);
        let out = format!("{}/{}.png", self.config.output_dir, ts_fmt);
        tracing::debug!(path = %out, "Capturing screenshot");

        #[cfg(target_os = "macos")]
        {
            Command::new("screencapture")
                .arg("-x")
                .arg("-t")
                .arg("png")
                .arg(&out)
                .status()
                .map_err(LoggerError::Io)?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            let cmd = if cfg!(target_os = "linux") {
                "grim"
            } else {
                "screenshot.exe"
            };
            Command::new(cmd)
                .arg("-t")
                .arg("png")
                .arg(&out)
                .status()
                .map_err(LoggerError::Io)?;
        }

        let image_data = tokio::fs::read(&out).await.map_err(LoggerError::Io)?;

        if let Err(e) = tokio::fs::remove_file(&out).await {
            tracing::warn!(error = %e, "Failed to delete temporary screenshot");
        }

        Ok(image_data)
    }
}

#[async_trait]
impl DataLogger for ScreenLogger {
    type Config = ScreenConfig;

    fn new(config: ScreenConfig) -> Result<Self, LoggerError> {
        ScreenLogger::new(config)
    }

    fn setup(&self, config: ScreenConfig) -> Result<LoggerHandle, LoggerError> {
        let logger = Self::new(config)?;
        let join = tokio::spawn(async move {
            let task_result = logger.run().await;

            tracing::info!(result = ?task_result, "Background task finished");

            task_result
        });

        Ok(LoggerHandle { join })
    }

    async fn run(&self) -> Result<(), LoggerError> {
        RUNNING.store(true, Ordering::SeqCst);
        while RUNNING.load(Ordering::SeqCst) {
            self.log_data().await?;
            sleep(Duration::from_secs_f64(self.config.interval)).await;
        }
        Ok(())
    }

    fn stop(&self) {
        RUNNING.store(false, Ordering::SeqCst);
    }

    async fn log_data(&self) -> Result<Vec<u8>, LoggerError> {
        self.capture_screenshot_data().await
    }
}
