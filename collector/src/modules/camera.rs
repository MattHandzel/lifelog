use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use config::CameraConfig;
use lifelog_core::{LifelogError, Utc, Uuid};
use lifelog_types::{to_pb_ts, CameraFrame};
use prost::Message;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

#[cfg(target_os = "linux")]
use rscam::{Camera, Config};

#[cfg(target_os = "macos")]
use std::fs;
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(target_os = "macos")]
use tempfile::NamedTempFile;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct CameraDataSource {
    config: CameraConfig,
    pub buffer: Arc<DiskBuffer>,
}

impl CameraDataSource {
    pub fn new(config: CameraConfig) -> Result<Self, LifelogError> {
        let buffer_path = std::path::Path::new(&config.output_dir).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(CameraDataSource {
            config,
            buffer: Arc::new(buffer),
        })
    }
}

#[async_trait]
impl DataSource for CameraDataSource {
    type Config = CameraConfig;

    fn new(config: CameraConfig) -> Result<Self, LifelogError> {
        CameraDataSource::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(CameraBufferedSource {
            stream_id: "camera".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            return Err(LifelogError::AlreadyRunning);
        }

        tracing::info!("CameraDataSource: Starting data source task");
        RUNNING.store(true, Ordering::SeqCst);

        let source_clone = self.clone();

        let _join_handle = tokio::spawn(async move {
            let task_result = source_clone.run().await;
            tracing::info!(result = ?task_result, "CameraDataSource background task finished");
            task_result
        });

        let new_join_handle = tokio::spawn(async { Ok(()) });
        Ok(DataSourceHandle {
            join: new_join_handle,
        })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        RUNNING.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        #[cfg(target_os = "linux")]
        {
            let mut camera = Camera::new(&self.config.device).map_err(|e| {
                LifelogError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

            let camera_config = Config {
                interval: (1, self.config.fps),
                format: b"MJPG",
                resolution: (self.config.resolution_x, self.config.resolution_y),
                ..Default::default()
            };

            camera.start(&camera_config).map_err(|e| {
                LifelogError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

            while RUNNING.load(Ordering::SeqCst) {
                match camera.capture() {
                    Ok(frame) => {
                        let timestamp = to_pb_ts(Utc::now());
                        let pb_frame = CameraFrame {
                            uuid: Uuid::new_v4().to_string(),
                            timestamp,
                            width: frame.resolution.0,
                            height: frame.resolution.1,
                            image_bytes: frame.to_vec(),
                            mime_type: "image/jpeg".to_string(),
                            device: self.config.device.clone(),
                            t_device: timestamp,
                            t_canonical: timestamp,
                            t_end: timestamp,
                            ..Default::default()
                        };

                        let mut buf = Vec::new();
                        if let Err(e) = pb_frame.encode(&mut buf) {
                            tracing::error!("Failed to encode CameraFrame: {}", e);
                        } else if let Err(e) = self.buffer.append(&buf).await {
                            tracing::error!("Failed to append CameraFrame to buffer: {}", e);
                        } else {
                            tracing::debug!("Stored camera frame in WAL");
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to capture frame: {}", e);
                    }
                }
                sleep(Duration::from_secs_f64(self.config.interval)).await;
            }
        }

        #[cfg(target_os = "macos")]
        {
            while RUNNING.load(Ordering::SeqCst) {
                // macOS implementation using imagesnap
                if let Ok(temp_file) = NamedTempFile::new() {
                    let temp_path = temp_file.path().to_string_lossy().to_string();

                    let output = Command::new("imagesnap")
                        .arg("-w")
                        .arg("0.5")
                        .arg(&temp_path)
                        .output();

                    match output {
                        Ok(out) if out.status.success() => {
                            if let Ok(data) = fs::read(&temp_path) {
                                if !data.is_empty() {
                                    let timestamp = to_pb_ts(Utc::now());
                                    let pb_frame = CameraFrame {
                                        uuid: Uuid::new_v4().to_string(),
                                        timestamp,
                                        width: 0, // Unknown without decoding
                                        height: 0,
                                        image_bytes: data,
                                        mime_type: "image/jpeg".to_string(),
                                        device: "imagesnap".to_string(),
                                        t_device: timestamp,
                                        t_canonical: timestamp,
                                        t_end: timestamp,
                                        ..Default::default()
                                    };

                                    let mut buf = Vec::new();
                                    if let Err(e) = pb_frame.encode(&mut buf) {
                                        tracing::error!("Failed to encode CameraFrame: {}", e);
                                    } else if let Err(e) = self.buffer.append(&buf).await {
                                        tracing::error!(
                                            "Failed to append CameraFrame to buffer: {}",
                                            e
                                        );
                                    } else {
                                        tracing::debug!("Stored camera frame in WAL");
                                    }
                                }
                            }
                        }
                        _ => {
                            tracing::error!("Failed to run imagesnap");
                        }
                    }
                }

                sleep(Duration::from_secs_f64(self.config.interval)).await;
            }
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            tracing::warn!("Camera capture not supported on this OS");
            sleep(Duration::from_secs(10)).await;
        }

        Ok(())
    }

    fn is_running(&self) -> bool {
        RUNNING.load(Ordering::SeqCst)
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}

pub struct CameraBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for CameraBufferedSource {
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
