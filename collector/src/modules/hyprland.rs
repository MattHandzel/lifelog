use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use config::HyprlandConfig;
use hyprland::data::{Clients, CursorPosition, Devices, Monitors, Workspace, Workspaces};
use hyprland::shared::HyprData;
use hyprland::shared::HyprDataActive;
use lifelog_core::{LifelogError, Utc, Uuid};
use lifelog_types::{
    to_pb_ts, HyprClient, HyprCursor, HyprDevice, HyprMonitor, HyprWorkspace, HyprlandFrame,
};
use prost::Message;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct HyprlandDataSource {
    config: HyprlandConfig,
    pub buffer: Arc<DiskBuffer>,
}

impl HyprlandDataSource {
    pub fn new(config: HyprlandConfig) -> Result<Self, LifelogError> {
        let buffer_path = std::path::Path::new(&config.output_dir).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(HyprlandDataSource {
            config,
            buffer: Arc::new(buffer),
        })
    }

    async fn capture_frame(&self) -> Result<HyprlandFrame, LifelogError> {
        let timestamp = to_pb_ts(Utc::now());
        let mut frame = HyprlandFrame {
            uuid: Uuid::new_v4().to_string(),
            timestamp,
            t_device: timestamp,
            t_canonical: timestamp,
            t_end: timestamp,
            ..Default::default()
        };

        if self.config.log_active_monitor {
            if let Ok(monitors) = Monitors::get() {
                frame.monitors = monitors
                    .into_iter()
                    .map(|m| HyprMonitor {
                        id: m.id as i32,
                        name: m.name,
                        description: m.description,
                        width: m.width as i32,
                        height: m.height as i32,
                        refresh_rate: m.refresh_rate,
                        x: m.x,
                        y: m.y,
                        workspace_id: m.active_workspace.id,
                        workspace_name: m.active_workspace.name,
                        scale: m.scale,
                        focused: m.focused,
                    })
                    .collect();
            }
        }

        if self.config.log_workspace {
            if let Ok(workspaces) = Workspaces::get() {
                frame.workspaces = workspaces
                    .into_iter()
                    .map(|w| HyprWorkspace {
                        id: w.id,
                        name: w.name,
                        monitor: w.monitor,
                        monitor_id: w.monitor_id as i32,
                        windows: w.windows as i32,
                        fullscreen: w.fullscreen,
                        last_window: w.last_window.to_string(),
                        last_window_title: w.last_window_title,
                    })
                    .collect();
            }
            if let Ok(aw) = Workspace::get_active() {
                frame.active_workspace = Some(HyprWorkspace {
                    id: aw.id,
                    name: aw.name,
                    monitor: aw.monitor,
                    monitor_id: aw.monitor_id as i32,
                    windows: aw.windows as i32,
                    fullscreen: aw.fullscreen,
                    last_window: aw.last_window.to_string(),
                    last_window_title: aw.last_window_title,
                });
            }
        }

        if self.config.log_clients {
            if let Ok(clients) = Clients::get() {
                frame.clients = clients
                    .into_iter()
                    .map(|c| HyprClient {
                        address: c.address.to_string(),
                        x: c.at.0 as i32,
                        y: c.at.1 as i32,
                        width: c.size.0 as i32,
                        height: c.size.1 as i32,
                        workspace_id: c.workspace.id,
                        workspace_name: c.workspace.name,
                        floating: c.floating,
                        fullscreen: format!("{:?}", c.fullscreen),
                        monitor: c.monitor as i32,
                        title: c.title,
                        class: c.class,
                        pid: c.pid,
                        pinned: c.pinned,
                        mapped: c.mapped,
                    })
                    .collect();
            }
        }

        if self.config.log_devices {
            if let Ok(devices) = Devices::get() {
                let mut pb_devices = Vec::new();
                for m in devices.mice {
                    pb_devices.push(HyprDevice {
                        r#type: "mouse".to_string(),
                        name: m.name,
                        address: m.address.to_string(),
                    });
                }
                for k in devices.keyboards {
                    pb_devices.push(HyprDevice {
                        r#type: "keyboard".to_string(),
                        name: k.name,
                        address: k.address.to_string(),
                    });
                }
                for t in devices.tablets {
                    pb_devices.push(HyprDevice {
                        r#type: "tablet".to_string(),
                        name: t.name.unwrap_or_default(),
                        address: t.address.to_string(),
                    });
                }
                frame.devices = pb_devices;
            }
        }

        if let Ok(pos) = CursorPosition::get() {
            frame.cursor = Some(HyprCursor {
                x: pos.x as f64,
                y: pos.y as f64,
            });
        }

        Ok(frame)
    }
}

#[async_trait]
impl DataSource for HyprlandDataSource {
    type Config = HyprlandConfig;

    fn new(config: HyprlandConfig) -> Result<Self, LifelogError> {
        HyprlandDataSource::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(HyprlandBufferedSource {
            stream_id: "hyprland".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            return Err(LifelogError::AlreadyRunning);
        }

        tracing::info!("HyprlandDataSource: Starting data source task");
        RUNNING.store(true, Ordering::SeqCst);

        let source_clone = self.clone();

        let _join_handle = tokio::spawn(async move {
            let task_result = source_clone.run().await;
            tracing::info!(result = ?task_result, "HyprlandDataSource background task finished");
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
        while RUNNING.load(Ordering::SeqCst) {
            match self.capture_frame().await {
                Ok(frame) => {
                    let mut buf = Vec::new();
                    if let Err(e) = frame.encode(&mut buf) {
                        tracing::error!("Failed to encode HyprlandFrame: {}", e);
                    } else if let Err(e) = self.buffer.append(&buf).await {
                        tracing::error!("Failed to append HyprlandFrame to buffer: {}", e);
                    } else {
                        tracing::debug!("Stored hyprland frame in WAL");
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to capture hyprland frame: {}", e);
                }
            }
            sleep(Duration::from_secs_f64(self.config.interval)).await;
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

pub struct HyprlandBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for HyprlandBufferedSource {
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
