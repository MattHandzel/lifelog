use crate::logger::*;
use crate::setup;
use async_trait::async_trait;
use config::HyprlandConfig;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use utils::current_timestamp;

use hyprland::data::{Clients, CursorPosition, Devices, Monitors, Workspace, Workspaces};

use hyprland::shared::HyprData;
use hyprland::shared::HyprDataActive;

pub struct HyprlandLogger {
    config: HyprlandConfig,
    running_flag: Arc<AtomicBool>,
}

#[async_trait]
impl DataLogger for HyprlandLogger {
    type Config = HyprlandConfig;

    fn setup(&self, config: HyprlandConfig) -> Result<LoggerHandle, LoggerError> {
        setup::setup_hyprland_db(Path::new(&self.config.output_dir));
            let logger = Self::new(config)?;
            let join = tokio::spawn(async move {
    
                let task_result = logger.run().await;
    
                tracing::debug!(?task_result, "Background task finished");
    
                task_result
            });
    
            Ok(LoggerHandle { join })
        }

    async fn run(&self) -> Result<(), LoggerError> {
        self.running_flag.store(true, Ordering::SeqCst);

        while self.running_flag.load(Ordering::SeqCst) {
            let _timestamp = current_timestamp();
            self.log_data().await?;

            sleep(Duration::from_secs_f64(self.config.interval)).await;
        }
        Ok(())
    }

    fn stop(&self) {
        self.running_flag.store(false, Ordering::SeqCst);
    }

    async fn log_data(&self) -> Result<(), LoggerError> {
        let timestamp = current_timestamp();
        self.log_hypr_data(timestamp)
            .await
            .map_err(|e| LoggerError::Generic(e.to_string()))?;
        Ok(())
    }

    // TODO: I could set up the connection here...
    fn new(config: Self::Config) -> Result<Self, LoggerError> {
        Ok(Self {
            config,
            running_flag: Arc::new(AtomicBool::new(false)),
        })
    }
}

// Implementation details
impl HyprlandLogger {
    pub fn setup(&self) -> Result<LoggerHandle, LoggerError> {
        DataLogger::setup(self, self.config.clone())
    }

    async fn log_hypr_data(&self, timestamp: f64) -> Result<(), Box<dyn std::error::Error>> {
        {
            let conn = setup::setup_hyprland_db(Path::new(&self.config.output_dir))
                .expect("Failed to set up Hyprland database");

            loop {
                if self.config.log_active_monitor {
                    log_monitors(&conn, timestamp);
                }

                if self.config.log_workspace {
                    log_workspaces(&conn, timestamp);
                }

                if self.config.log_clients {
                    log_clients(&conn, timestamp);
                }

                if self.config.log_devices {
                    log_devices(&conn, timestamp);
                }

                log_cursor_position(&conn, timestamp);

                sleep(Duration::from_secs_f64(self.config.interval)).await;
            }
        }
        Ok(())
    }
}

fn log_monitors(conn: &Connection, timestamp: f64) {
    if let Ok(monitors) = Monitors::get() {
        for monitor in monitors {
            conn.execute(
                "INSERT INTO monitors VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    timestamp,
                    monitor.id as i64,
                    monitor.name,
                    monitor.description,
                    monitor.width,
                    monitor.height,
                    monitor.refresh_rate,
                    monitor.x,
                    monitor.y,
                    monitor.active_workspace.id as i64,
                    monitor.active_workspace.name,
                    monitor.scale,
                    monitor.focused
                ],
            ).unwrap();
        }
    }
}

fn log_workspaces(conn: &Connection, timestamp: f64) {
    if let Ok(workspaces) = Workspaces::get() {
        for workspace in workspaces {
            conn.execute(
                "INSERT INTO workspaces VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    timestamp,
                    workspace.id as i64,
                    workspace.name,
                    workspace.monitor,
                    workspace.monitor_id as i64,
                    workspace.windows,
                    workspace.fullscreen,
                    workspace.last_window.to_string(),
                    workspace.last_window_title
                ],
            )
            .unwrap();
        }
    }
    if let Ok(active_workspaces) = Workspace::get_active() {
        conn.execute(
            "INSERT INTO activeworkspace VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                timestamp,
                active_workspaces.id as i64,
                active_workspaces.name,
                active_workspaces.monitor,
                active_workspaces.monitor_id as i64,
                active_workspaces.windows,
                active_workspaces.fullscreen,
                active_workspaces.last_window.to_string(),
                active_workspaces.last_window_title
            ],
        )
        .unwrap();
    }
}

fn log_clients(conn: &Connection, timestamp: f64) {
    if let Ok(clients) = Clients::get() {
        for client in clients {
            conn.execute(
                "INSERT INTO clients VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    timestamp,
                    client.address.to_string(),
                    client.at.0,
                    client.at.1,
                    client.size.0,
                    client.size.1,
                    client.workspace.id,
                    client.workspace.name,
                    client.floating,
                    format!("{:?}", client.fullscreen),
                    client.monitor as i64,
                    client.title,
                    client.class,
                    client.pid,
                    client.pinned,
                    client.mapped,
                    client.focus_history_id
                ],
            ).unwrap();
        }
    }
}

fn log_devices(conn: &Connection, timestamp: f64) {
    if let Ok(devices) = Devices::get() {
        // Log mice
        for mouse in devices.mice {
            conn.execute(
                "INSERT INTO devices VALUES (?1, 'mouse', ?2, ?3)",
                params![timestamp, mouse.name, mouse.address.to_string()],
            )
            .unwrap();
        }

        // Log keyboards
        for keyboard in devices.keyboards {
            conn.execute(
                "INSERT INTO devices VALUES (?1, 'keyboard', ?2, ?3)",
                params![timestamp, keyboard.name, keyboard.address.to_string()],
            )
            .unwrap();
        }
        //pub tablets: Vec<Tablet>,
        for tablet in devices.tablets {
            conn.execute(
                "INSERT INTO devices VALUES (?1, 'tablet', ?2, ?3)",
                params![timestamp, tablet.name, tablet.address.to_string()],
            )
            .unwrap();
        }
    }
}

fn log_cursor_position(conn: &Connection, timestamp: f64) {
    if let Ok(pos) = CursorPosition::get() {
        conn.execute(
            "INSERT INTO cursor_positions VALUES (?1, ?2, ?3)",
            params![timestamp, pos.x, pos.y],
        )
        .unwrap();
    }
}
