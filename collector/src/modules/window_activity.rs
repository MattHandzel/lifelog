use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use config::WindowActivityConfig;
use lifelog_core::{LifelogError, Uuid};
use lifelog_types::{to_pb_ts, RecordType, WindowActivityFrame};
use prost::Message;
use serde::Deserialize;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct WindowActivityDataSource {
    config: WindowActivityConfig,
    pub buffer: Arc<DiskBuffer>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WindowInfo {
    application: String,
    window_title: String,
}

#[derive(Debug, Clone)]
struct ActiveWindowSpan {
    info: WindowInfo,
    start: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backend {
    Auto,
    X11,
    Hyprctl,
}

impl Backend {
    fn parse(s: &str) -> Result<Self, LifelogError> {
        match s.trim().to_ascii_lowercase().as_str() {
            "" | "auto" => Ok(Self::Auto),
            "x11" => Ok(Self::X11),
            "hyprctl" => Ok(Self::Hyprctl),
            other => Err(LifelogError::Validation {
                field: "window_activity.backend".to_string(),
                reason: format!("unsupported backend: {other} (expected auto/x11/hyprctl)"),
            }),
        }
    }
}

impl WindowActivityDataSource {
    pub fn new(config: WindowActivityConfig) -> Result<Self, LifelogError> {
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

    fn pick_backends(&self) -> Result<Vec<Backend>, LifelogError> {
        let b = Backend::parse(&self.config.backend)?;
        Ok(match b {
            Backend::Auto => vec![Backend::Hyprctl, Backend::X11],
            Backend::Hyprctl => vec![Backend::Hyprctl],
            Backend::X11 => vec![Backend::X11],
        })
    }

    fn read_active_window(&self) -> Result<WindowInfo, LifelogError> {
        let backends = self.pick_backends()?;
        let mut last_err: Option<LifelogError> = None;

        for b in backends {
            let res = match b {
                Backend::Hyprctl => self.read_active_window_hyprctl(),
                Backend::X11 => self.read_active_window_x11_best_effort(),
                Backend::Auto => unreachable!("Auto expanded by pick_backends"),
            };

            match res {
                Ok(info) => return Ok(info),
                Err(e) => last_err = Some(e),
            }
        }

        Err(last_err.unwrap_or(LifelogError::Validation {
            field: "window_activity".to_string(),
            reason: "failed to read active window".to_string(),
        }))
    }

    fn read_active_window_hyprctl(&self) -> Result<WindowInfo, LifelogError> {
        // Hyprland: hyprctl is the most portable interface; parse a minimal JSON payload.
        // Example keys include "class" and "title".
        let out = Command::new("hyprctl")
            .args(["activewindow", "-j"])
            .output()
            .map_err(LifelogError::Io)?;
        if !out.status.success() {
            return Err(LifelogError::Validation {
                field: "window_activity.hyprctl".to_string(),
                reason: "hyprctl activewindow failed".to_string(),
            });
        }

        #[derive(Debug, Deserialize)]
        struct HyprActiveWindow {
            #[serde(default)]
            class: String,
            #[serde(default)]
            title: String,
        }

        let parsed: HyprActiveWindow = serde_json::from_slice(&out.stdout)
            .map_err(|e| LifelogError::Database(e.to_string()))?;

        let application = parsed.class.trim().to_string();
        let window_title = parsed.title.trim().to_string();
        if application.is_empty() && window_title.is_empty() {
            return Err(LifelogError::Validation {
                field: "window_activity.hyprctl".to_string(),
                reason: "hyprctl returned empty active window".to_string(),
            });
        }

        Ok(WindowInfo {
            application,
            window_title,
        })
    }

    fn read_active_window_x11_best_effort(&self) -> Result<WindowInfo, LifelogError> {
        // Prefer xdotool (simpler parsing), fall back to xprop.
        if let Ok(info) = self.read_active_window_xdotool() {
            return Ok(info);
        }
        self.read_active_window_xprop()
    }

    fn read_active_window_xdotool(&self) -> Result<WindowInfo, LifelogError> {
        let class_out = Command::new("xdotool")
            .args(["getactivewindow", "getwindowclassname"])
            .output()
            .map_err(LifelogError::Io)?;
        if !class_out.status.success() {
            return Err(LifelogError::Validation {
                field: "window_activity.xdotool".to_string(),
                reason: "xdotool getwindowclassname failed".to_string(),
            });
        }

        let title_out = Command::new("xdotool")
            .args(["getactivewindow", "getwindowname"])
            .output()
            .map_err(LifelogError::Io)?;
        if !title_out.status.success() {
            return Err(LifelogError::Validation {
                field: "window_activity.xdotool".to_string(),
                reason: "xdotool getwindowname failed".to_string(),
            });
        }

        let application = String::from_utf8_lossy(&class_out.stdout)
            .trim()
            .to_string();
        let window_title = String::from_utf8_lossy(&title_out.stdout)
            .trim()
            .to_string();
        if application.is_empty() && window_title.is_empty() {
            return Err(LifelogError::Validation {
                field: "window_activity.xdotool".to_string(),
                reason: "xdotool returned empty active window".to_string(),
            });
        }
        Ok(WindowInfo {
            application,
            window_title,
        })
    }

    fn read_active_window_xprop(&self) -> Result<WindowInfo, LifelogError> {
        let win_out = Command::new("xprop")
            .args(["-root", "_NET_ACTIVE_WINDOW"])
            .output()
            .map_err(LifelogError::Io)?;
        if !win_out.status.success() {
            return Err(LifelogError::Validation {
                field: "window_activity.xprop".to_string(),
                reason: "xprop -root _NET_ACTIVE_WINDOW failed".to_string(),
            });
        }

        let s = String::from_utf8_lossy(&win_out.stdout);
        // Typical: `_NET_ACTIVE_WINDOW(WINDOW): window id # 0x4a0000e`
        let win_id = s
            .split_whitespace()
            .find(|tok| tok.starts_with("0x"))
            .ok_or_else(|| LifelogError::Validation {
                field: "window_activity.xprop".to_string(),
                reason: "failed to parse active window id".to_string(),
            })?
            .to_string();

        let props_out = Command::new("xprop")
            .args(["-id", &win_id, "WM_CLASS", "_NET_WM_NAME"])
            .output()
            .map_err(LifelogError::Io)?;
        if !props_out.status.success() {
            return Err(LifelogError::Validation {
                field: "window_activity.xprop".to_string(),
                reason: "xprop -id ... WM_CLASS/_NET_WM_NAME failed".to_string(),
            });
        }
        let props = String::from_utf8_lossy(&props_out.stdout);

        let mut application = String::new();
        let mut window_title = String::new();
        for line in props.lines() {
            if line.starts_with("WM_CLASS") {
                // `WM_CLASS(STRING) = "Navigator", "firefox"`
                // Prefer the second token if present.
                let quoted: Vec<String> = line
                    .split('"')
                    .skip(1)
                    .step_by(2)
                    .map(|s| s.to_string())
                    .collect();
                if quoted.len() >= 2 {
                    application = quoted[1].trim().to_string();
                } else if quoted.len() == 1 {
                    application = quoted[0].trim().to_string();
                }
            } else if line.contains("_NET_WM_NAME") {
                // `_NET_WM_NAME(UTF8_STRING) = "Some Title"`
                let quoted: Vec<String> = line
                    .split('"')
                    .skip(1)
                    .step_by(2)
                    .map(|s| s.to_string())
                    .collect();
                if let Some(first) = quoted.first() {
                    window_title = first.trim().to_string();
                }
            }
        }

        if application.is_empty() && window_title.is_empty() {
            return Err(LifelogError::Validation {
                field: "window_activity.xprop".to_string(),
                reason: "xprop returned empty WM_CLASS/_NET_WM_NAME".to_string(),
            });
        }

        Ok(WindowInfo {
            application,
            window_title,
        })
    }

    async fn emit_span(
        &self,
        span: &ActiveWindowSpan,
        end: DateTime<Utc>,
    ) -> Result<(), LifelogError> {
        let start_ts = to_pb_ts(span.start);
        let end_ts = to_pb_ts(end);
        let duration = (end - span.start).num_milliseconds().max(0) as f32 / 1000.0;

        let frame = WindowActivityFrame {
            uuid: Uuid::new_v4().to_string(),
            timestamp: start_ts,
            application: span.info.application.clone(),
            window_title: span.info.window_title.clone(),
            focused: true,
            duration_secs: duration,
            t_device: start_ts,
            t_canonical: start_ts,
            t_end: end_ts,
            record_type: RecordType::Interval as i32,
            ..Default::default()
        };

        let mut buf = Vec::new();
        frame
            .encode(&mut buf)
            .map_err(|e| LifelogError::Database(e.to_string()))?;
        self.buffer.append(&buf).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        Ok(())
    }
}

#[async_trait]
impl DataSource for WindowActivityDataSource {
    type Config = WindowActivityConfig;

    fn new(config: WindowActivityConfig) -> Result<Self, LifelogError> {
        Self::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(WindowActivityBufferedSource {
            stream_id: "window_activity".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            tracing::warn!("WindowActivityDataSource: Start called but task is already running.");
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
        let mut current: Option<ActiveWindowSpan> = None;

        while RUNNING.load(Ordering::SeqCst) {
            let now = Utc::now();
            match self.read_active_window() {
                Ok(info) => {
                    let changed = current.as_ref().map(|c| c.info != info).unwrap_or(true);

                    if changed {
                        if let Some(prev) = current.take() {
                            if let Err(e) = self.emit_span(&prev, now).await {
                                tracing::error!(error = %e, "Failed to store WindowActivityFrame");
                            }
                        }

                        current = Some(ActiveWindowSpan { info, start: now });
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, "Window activity read failed");
                }
            }

            let interval = if self.config.interval > 0.0 {
                self.config.interval
            } else {
                1.0
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

pub struct WindowActivityBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for WindowActivityBufferedSource {
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
