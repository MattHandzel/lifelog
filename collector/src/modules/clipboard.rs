use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use chrono::Utc;
use config::ClipboardConfig;
use lifelog_core::{LifelogError, Uuid};
use lifelog_types::{to_pb_ts, ClipboardFrame};
use prost::Message;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct ClipboardDataSource {
    config: ClipboardConfig,
    pub buffer: Arc<DiskBuffer>,
}

impl ClipboardDataSource {
    pub fn new(config: ClipboardConfig) -> Result<Self, LifelogError> {
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

    fn read_clipboard_text(&self) -> Result<String, LifelogError> {
        // Prefer Wayland, then fall back to X11 tools. These are best-effort and are expected
        // to fail on systems where the tool isn't installed or a display isn't available.
        let candidates: &[(&str, &[&str])] = &[
            ("wl-paste", &["--no-newline"]),
            ("xclip", &["-o", "-selection", "clipboard"]),
            ("xsel", &["--clipboard", "--output"]),
        ];

        for (bin, args) in candidates {
            let out = Command::new(bin).args(*args).output();
            let Ok(out) = out else { continue };
            if !out.status.success() {
                continue;
            }
            let mut s = String::from_utf8_lossy(&out.stdout).to_string();
            // Normalize CRLF and trailing newlines so dedupe is stable.
            if s.ends_with('\n') {
                while s.ends_with('\n') || s.ends_with('\r') {
                    s.pop();
                }
            }
            if s.is_empty() {
                continue;
            }
            if self.config.max_text_bytes > 0 && s.len() > self.config.max_text_bytes as usize {
                s.truncate(self.config.max_text_bytes as usize);
            }
            return Ok(s);
        }

        Err(LifelogError::Validation {
            field: "clipboard".to_string(),
            reason: "failed to read clipboard via wl-paste/xclip/xsel".to_string(),
        })
    }
}

#[async_trait]
impl DataSource for ClipboardDataSource {
    type Config = ClipboardConfig;

    fn new(config: ClipboardConfig) -> Result<Self, LifelogError> {
        Self::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(ClipboardBufferedSource {
            stream_id: "clipboard".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            tracing::warn!("ClipboardDataSource: Start called but task is already running.");
            return Err(LifelogError::AlreadyRunning);
        }

        RUNNING.store(true, Ordering::SeqCst);
        let source_clone = self.clone();
        let _join_handle = tokio::spawn(async move { source_clone.run().await });

        // Existing sources return a dummy handle; keep the convention.
        Ok(DataSourceHandle {
            join: tokio::spawn(async { Ok(()) }),
        })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        RUNNING.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        let mut last_text: Option<String> = None;

        while RUNNING.load(Ordering::SeqCst) {
            match self.read_clipboard_text() {
                Ok(text) => {
                    let changed = last_text.as_deref() != Some(text.as_str());
                    if changed {
                        let timestamp = to_pb_ts(Utc::now());
                        let frame = ClipboardFrame {
                            uuid: Uuid::new_v4().to_string(),
                            timestamp,
                            text: text.clone(),
                            mime_type: "text/plain".to_string(),
                            t_device: timestamp,
                            t_canonical: timestamp,
                            t_end: timestamp,
                            ..Default::default()
                        };

                        let mut buf = Vec::new();
                        if let Err(e) = frame.encode(&mut buf) {
                            tracing::error!("Failed to encode ClipboardFrame: {}", e);
                        } else if let Err(e) = self.buffer.append(&buf).await {
                            tracing::error!("Failed to append ClipboardFrame to buffer: {}", e);
                        } else {
                            tracing::debug!("Stored clipboard frame in WAL");
                            last_text = Some(text);
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, "Clipboard read failed");
                }
            }

            let interval = if self.config.interval > 0.0 {
                self.config.interval
            } else {
                2.0
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

pub struct ClipboardBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for ClipboardBufferedSource {
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
