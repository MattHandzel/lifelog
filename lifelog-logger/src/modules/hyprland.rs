use chrono::Local;
use rusqlite::{Connection, params};
use tokio::time::{sleep, Duration};
use crate::config::HyprlandConfig;
use crate::setup;
use std::path::Path;
use crate::prelude::*;

//use hyprland::data::*;
use hyprland::prelude::*;
use hyprland::data::{
    Monitors, Workspaces, Clients, Devices,
    CursorPosition, Workspace, Client
};



pub async fn start_logger(config: &HyprlandConfig) {
    let conn = setup::setup_hyprland_db(Path::new(&config.output_dir))
        .expect("Failed to set up Hyprland database");

    loop {

        // Fetch and log enabled data types
        if config.log_active_monitor {
            let timestamp = Local::now().timestamp() as f64 + 
                Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
            log_monitors(&conn, timestamp);
        }
        
        if config.log_workspace {
            let timestamp = Local::now().timestamp() as f64 + 
                Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
            log_workspaces(&conn, timestamp);
        }

        if config.log_clients {
            let timestamp = Local::now().timestamp() as f64 + 
                Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
            log_clients(&conn, timestamp);
        }

        if config.log_devices {
            let timestamp = Local::now().timestamp() as f64 + 
                Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
            log_devices(&conn, timestamp);
        }

        let timestamp = Local::now().timestamp() as f64 + 
            Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
        log_cursor_position(&conn, timestamp);

        sleep(Duration::from_secs_f64(config.interval)).await;
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
    if let Ok(workspaces) = Workspaces::get(){
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
            ).unwrap();
        }
    }
    if let Ok(active_workspaces) = Workspace::get_active(){
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
        ).unwrap();
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
            ).unwrap();
        }

        // Log keyboards
        for keyboard in devices.keyboards {
            conn.execute(
                "INSERT INTO devices VALUES (?1, 'keyboard', ?2, ?3)",
                params![timestamp, keyboard.name, keyboard.address.to_string()],
            ).unwrap();
        }
    //pub tablets: Vec<Tablet>,
        for tablet in devices.tablets {
            conn.execute(
            "INSERT INTO devices VALUES (?1, 'tablet', ?2, ?3)",
            params![timestamp, tablet.name, tablet.address.to_string()],
            ).unwrap();
        }
    }
}

fn log_cursor_position(conn: &Connection, timestamp: f64) {
    if let Ok(pos) = CursorPosition::get() {
        conn.execute(
            "INSERT INTO cursor_positions VALUES (?1, ?2, ?3)",
            params![timestamp, pos.x, pos.y],
        ).unwrap();
    }
}
