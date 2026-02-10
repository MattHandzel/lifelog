use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use chrono::Utc;
use config::MouseConfig;
use lifelog_core::{LifelogError, Uuid};
use lifelog_types::{to_pb_ts, MouseFrame};
use prost::Message;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

#[cfg(target_os = "linux")]
use hyprland::shared::HyprData;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct MouseDataSource {
    config: MouseConfig,
    pub buffer: Arc<DiskBuffer>,
}

impl MouseDataSource {
    pub fn new(config: MouseConfig) -> Result<Self, LifelogError> {
        let buffer_path = std::path::Path::new(&config.output_dir).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(Self {
            config,
            buffer: Arc::new(buffer),
        })
    }

    #[cfg(target_os = "linux")]
    fn get_cursor_pos(&self) -> Option<(f64, f64)> {
        // Best-effort: this works on Hyprland. On other compositors it will likely fail, but the
        // module should degrade quietly (Spec §1.3: quiet alerts).
        hyprland::data::CursorPosition::get()
            .ok()
            .map(|p| (p.x as f64, p.y as f64))
    }

    #[cfg(not(target_os = "linux"))]
    fn get_cursor_pos(&self) -> Option<(f64, f64)> {
        None
    }
}

#[async_trait]
impl DataSource for MouseDataSource {
    type Config = MouseConfig;

    fn new(config: MouseConfig) -> Result<Self, LifelogError> {
        Self::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(MouseBufferedSource {
            stream_id: "mouse".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            return Err(LifelogError::AlreadyRunning);
        }

        RUNNING.store(true, Ordering::SeqCst);
        let source_clone = self.clone();
        let _join_handle = tokio::spawn(async move { source_clone.run().await });

        Ok(DataSourceHandle {
            join: tokio::spawn(async { Ok(()) }),
        })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        RUNNING.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        let mut last_pos: Option<(f64, f64)> = None;
        let mut warned = false;

        while RUNNING.load(Ordering::SeqCst) {
            match self.get_cursor_pos() {
                Some((x, y)) => {
                    warned = false;
                    let changed = last_pos.map(|(lx, ly)| (lx != x) || (ly != y)).unwrap_or(true);
                    if changed {
                        let ts = to_pb_ts(Utc::now());
                        let frame = MouseFrame {
                            uuid: Uuid::new_v4().to_string(),
                            timestamp: ts,
                            x,
                            y,
                            button: lifelog_types::mouse_frame::MouseButton::None as i32,
                            pressed: false,
                            t_device: ts,
                            t_canonical: ts,
                            t_end: ts,
                            ..Default::default()
                        };

                        let mut buf = Vec::new();
                        if let Err(e) = frame.encode(&mut buf) {
                            tracing::error!(error = %e, "Failed to encode MouseFrame");
                        } else if let Err(e) = self.buffer.append(&buf).await {
                            tracing::error!(error = %e, "Failed to append MouseFrame to buffer");
                        } else {
                            last_pos = Some((x, y));
                        }
                    }
                }
                None => {
                    if !warned {
                        warned = true;
                        tracing::debug!("MouseDataSource: cursor position unavailable (non-Hyprland compositor or missing permissions)");
                    }
                }
            }

            let interval = if self.config.interval > 0.0 {
                self.config.interval
            } else {
                0.25
            };
            sleep(Duration::from_secs_f64(interval)).await;
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

pub struct MouseBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for MouseBufferedSource {
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
        })
    }
}
