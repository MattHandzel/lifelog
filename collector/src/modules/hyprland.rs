use crate::modules::polling_source::Capture;
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
use tokio::time::Duration;

pub struct HyprlandCapture {
    config: HyprlandConfig,
}

impl HyprlandCapture {
    fn build_frame(&self) -> HyprlandFrame {
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
                        monitor_id: w.monitor_id.unwrap_or(0) as i32,
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
                    monitor_id: aw.monitor_id.unwrap_or(0) as i32,
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
                        monitor: c.monitor.unwrap_or(0) as i32,
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

        frame
    }
}

#[async_trait]
impl Capture for HyprlandCapture {
    type Config = HyprlandConfig;

    fn from_config(config: HyprlandConfig) -> Result<Self, LifelogError> {
        Ok(Self { config })
    }

    fn output_dir(&self) -> &str {
        &self.config.output_dir
    }

    fn stream_id(&self) -> &str {
        "hyprland"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs_f64(self.config.interval)
    }

    async fn capture(&self) -> Result<Vec<Vec<u8>>, LifelogError> {
        let frame = self.build_frame();
        let mut buf = Vec::new();
        frame.encode(&mut buf).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to encode HyprlandFrame: {e}"),
            ))
        })?;
        Ok(vec![buf])
    }
}
