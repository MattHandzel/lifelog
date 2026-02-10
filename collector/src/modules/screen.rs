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
use lifelog_types::to_pb_ts;
use prost::Message;
use std::io::Cursor;

use std::sync::Arc;

use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use lifelog_core::LifelogError;
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct ScreenDataSource {
    config: ScreenConfig,
    logger: ScreenLogger,
    pub buffer: Arc<DiskBuffer>,
}

impl ScreenDataSource {
    pub fn new(config: ScreenConfig) -> Result<Self, LifelogError> {
        let logger = ScreenLogger::new(config.clone());
        let buffer_path = std::path::Path::new(&config.output_dir).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(ScreenDataSource {
            config,
            logger: logger?,
            buffer: Arc::new(buffer),
        })
    }

    pub async fn get_data(&mut self) -> Result<Vec<ScreenFrame>, LifelogError> {
        let (_, raw_items) = self.buffer.peek_chunk(100).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        let mut frames = Vec::new();
        for raw in raw_items {
            if let Ok(frame) = ScreenFrame::decode(raw.as_slice()) {
                frames.push(frame);
            }
        }
        Ok(frames)
    }

    pub async fn clear_buffer(&self) -> Result<(), LifelogError> {
        // Mark all current data as committed
        let start = self.buffer.get_committed_offset().await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        let size = self.buffer.get_uncommitted_size().await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        self.buffer.commit_offset(start + size).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        Ok(())
    }
}

#[async_trait]
impl DataSource for ScreenDataSource {
    type Config = ScreenConfig;

    fn new(config: ScreenConfig) -> Result<Self, LifelogError> {
        ScreenDataSource::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(ScreenBufferedSource {
            stream_id: "screen".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            tracing::warn!("ScreenDataSource: Start called but task is already running.");
            return Err(LifelogError::AlreadyRunning);
        }

        tracing::info!("ScreenDataSource: Starting data source task to store in WAL");
        RUNNING.store(true, Ordering::SeqCst);

        let source_clone = self.clone();

        let _join_handle = tokio::spawn(async move {
            let task_result = source_clone.run().await;
            tracing::info!(result = ?task_result, "ScreenDataSource (WAL) background task finished");
            task_result
        });

        tracing::info!("ScreenDataSource: Data source task (WAL) started successfully");
        let new_join_handle = tokio::spawn(async { Ok(()) });
        Ok(DataSourceHandle {
            join: new_join_handle,
        })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        RUNNING.store(false, Ordering::SeqCst);
        // FIXME, actually implmenet stop handles
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        while RUNNING.load(Ordering::SeqCst) {
            match self.logger.log_data().await {
                Ok(image_data_bytes) => {
                    let ts = Utc::now();

                    let img = ImageReader::new(Cursor::new(&image_data_bytes))
                        .with_guessed_format()
                        .map_err(|e| {
                            LifelogError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                        })?
                        .decode()
                        .map_err(|e| {
                            LifelogError::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
                        })?;

                    let (width, height) = img.dimensions();

                    let timestamp = to_pb_ts(ts);
                    let captured = ScreenFrame {
                        uuid: Uuid::new_v4().to_string(), //use v6
                        width,
                        height,
                        image_bytes: image_data_bytes,
                        timestamp,
                        mime_type: "image/png".to_string(),
                        t_device: timestamp,
                        t_canonical: timestamp,
                        t_end: timestamp,
                        ..Default::default()
                    };

                    let mut buf = Vec::new();
                    captured.encode(&mut buf).map_err(|e| {
                        LifelogError::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Prost encode error: {}", e),
                        ))
                    })?;

                    self.buffer.append(&buf).await.map_err(|e| {
                        LifelogError::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        ))
                    })?;

                    tracing::debug!("ScreenDataSource: Stored screen capture in WAL");
                }
                Err(e) => {
                    tracing::error!(error = %e, "ScreenDataSource: Failed to capture screen data for WAL store");
                }
            }
            sleep(Duration::from_secs_f64(self.config.interval)).await;
        }
        tracing::info!("ScreenDataSource: WAL run loop finished");
        Ok(())
    }

    fn is_running(&self) -> bool {
        RUNNING.load(Ordering::SeqCst)
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}

pub struct ScreenBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for ScreenBufferedSource {
    fn stream_id(&self) -> String {
        self.stream_id.clone()
    }

    async fn peek_upload_batch(
        &self,
        max_items: usize,
    ) -> Result<(u64, Vec<Vec<u8>>), LifelogError> {
        let (next_offset, raws) = self.buffer.peek_chunk(max_items).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok((next_offset, raws))
    }

    async fn commit_upload(&self, offset: u64) -> Result<(), LifelogError> {
        self.buffer.commit_offset(offset).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct ScreenLogger {
    config: ScreenConfig,
}

impl ScreenLogger {
    pub fn new(config: ScreenConfig) -> Result<Self, LifelogError> {
        Ok(ScreenLogger { config })
    }

    pub fn setup(&self) -> Result<LoggerHandle, LifelogError> {
        DataLogger::setup(self, self.config.clone())
    }

    async fn capture_screenshot_data(&self) -> Result<Vec<u8>, LifelogError> {
        // let temp_file = NamedTempFile::new_in(env::temp_dir())?.into_temp_path();
        // let temp_file_path_str = temp_file.to_str().ok_or_else(|| LifelogError::Io(std::io::Error::new(std::io::ErrorKind::Other, "Invalid temp file path")))?;

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
                .map_err(LifelogError::Io)?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            Command::new(&self.config.program)
                .arg("-t")
                .arg("png")
                .arg(&out)
                .status()
                .map_err(LifelogError::Io)?;
        }

        let image_data = tokio::fs::read(&out).await.map_err(LifelogError::Io)?;

        if let Err(e) = tokio::fs::remove_file(&out).await {
            tracing::warn!(error = %e, "Failed to delete temporary screenshot");
        }

        Ok(image_data)
    }
}

#[async_trait]
impl DataLogger for ScreenLogger {
    type Config = ScreenConfig;

    fn new(config: ScreenConfig) -> Result<Self, LifelogError> {
        ScreenLogger::new(config)
    }

    fn setup(&self, config: ScreenConfig) -> Result<LoggerHandle, LifelogError> {
        let logger = Self::new(config)?;
        let join = tokio::spawn(async move {
            let task_result = logger.run().await;

            tracing::info!(result = ?task_result, "Background task finished");

            task_result
        });

        Ok(LoggerHandle { join })
    }

    async fn run(&self) -> Result<(), LifelogError> {
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

    async fn log_data(&self) -> Result<Vec<u8>, LifelogError> {
        self.capture_screenshot_data().await
    }
}
