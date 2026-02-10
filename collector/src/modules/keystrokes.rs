use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use chrono::Utc;
use config::KeyboardConfig;
use lifelog_core::{LifelogError, Uuid};
use lifelog_types::{to_pb_ts, KeystrokeFrame, RecordType};
use prost::Message;
use rdev::{listen, Event, EventType, Key};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct KeystrokesDataSource {
    config: KeyboardConfig,
    pub buffer: Arc<DiskBuffer>,
}

impl KeystrokesDataSource {
    pub fn new(config: KeyboardConfig) -> Result<Self, LifelogError> {
        let out_dir = std::path::Path::new(&config.output_dir);
        let _ = std::fs::create_dir_all(out_dir);
        let buffer_path = out_dir.join("buffer");

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
}

#[derive(Debug, Clone)]
struct KeyEvent {
    at: chrono::DateTime<chrono::Utc>,
    key: Key,
}

fn spawn_rdev_listener(tx: mpsc::Sender<KeyEvent>) {
    std::thread::spawn(move || {
        let callback = move |event: Event| {
            let EventType::KeyPress(key) = event.event_type else {
                return;
            };

            // `rdev` doesn't provide a reliable event timestamp across platforms; use device now.
            let _ = tx.blocking_send(KeyEvent {
                at: Utc::now(),
                key,
            });
        };

        if let Err(e) = listen(callback) {
            tracing::warn!(error = ?e, "KeystrokesDataSource: input listener failed (permissions? unsupported display server?)");
        }
    });
}

#[async_trait]
impl DataSource for KeystrokesDataSource {
    type Config = KeyboardConfig;

    fn new(config: KeyboardConfig) -> Result<Self, LifelogError> {
        Self::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(KeystrokesBufferedSource {
            stream_id: "keystrokes".to_string(),
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
        // This is intentionally best-effort and disabled by default (see SPEC.md §3.1 / §12.4).
        tracing::warn!(
            "KeystrokesDataSource: enabled. This records key events and can capture sensitive data."
        );

        let (tx, mut rx) = mpsc::channel::<KeyEvent>(2048);
        spawn_rdev_listener(tx);

        // Avoid tight loop if the listener fails immediately.
        let mut idle_backoff = Duration::from_millis(10);

        while RUNNING.load(Ordering::SeqCst) {
            let evt = match tokio::time::timeout(Duration::from_millis(250), rx.recv()).await {
                Ok(Some(e)) => {
                    idle_backoff = Duration::from_millis(10);
                    e
                }
                Ok(None) => {
                    tracing::warn!("KeystrokesDataSource: listener channel closed; no more events");
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                Err(_) => {
                    sleep(idle_backoff).await;
                    idle_backoff = (idle_backoff * 2).min(Duration::from_secs(2));
                    continue;
                }
            };

            let ts = to_pb_ts(evt.at);
            let frame = KeystrokeFrame {
                uuid: Uuid::new_v4().to_string(),
                timestamp: ts,
                // Minimal policy: record key identity, not reconstructed text.
                text: format!("{:?}", evt.key),
                application: String::new(),
                window_title: String::new(),
                t_device: ts,
                t_canonical: ts,
                t_end: ts,
                record_type: RecordType::Point as i32,
                ..Default::default()
            };

            let mut buf = Vec::new();
            if let Err(e) = frame.encode(&mut buf) {
                tracing::error!(error = %e, "KeystrokesDataSource: failed to encode KeystrokeFrame");
                continue;
            }

            if let Err(e) = self.buffer.append(&buf).await {
                tracing::error!(error = %e, "KeystrokesDataSource: failed to append to buffer");
            }
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

pub struct KeystrokesBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for KeystrokesBufferedSource {
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
