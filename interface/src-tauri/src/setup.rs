use crate::config::Config;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::{ProcessExt, System, SystemExt};

// Ensures the directory exists, creating it if necessary.
pub fn ensure_directory(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn initialize_project(config: &Config) -> std::io::Result<()> {
    // TODO: Check to see if all of these things exist or not
    // TODO: These should be moved inside their respective loggers

    ensure_directory(Path::new(&config.screen.output_dir))?;
    ensure_directory(Path::new(&config.system_performance.output_dir))?;
    ensure_directory(Path::new(&config.ambient.output_dir))?;
    ensure_directory(Path::new(&config.weather.output_dir))?;
    ensure_directory(Path::new(&config.audio.output_dir))?;
    ensure_directory(Path::new(&config.geolocation.output_dir))?;
    ensure_directory(Path::new(&config.wifi.output_dir))?;
    ensure_directory(Path::new(&config.camera.output_dir))?;
    ensure_directory(Path::new(&config.microphone.output_dir))?;
    ensure_directory(Path::new(&config.input_logger.output_dir))?;

    //let keyboard_db = setup_keyboard_db(output_dir)?;
    //let mouse_db = setup_mouse_db(output_dir)?;
    Ok(())
}

pub fn is_already_running(process_name: &str) -> bool {
    let system = System::new_all();
    let mut process_count = 0;
    for process in system.processes_by_name(process_name) {
        if process.name() == process_name {
            process_count += 1;
            if process_count > 1 {
                return true;
            }
        }
    }
    false
}

/// Initializes the SQLite database and creates tables if they don't exist.
pub fn initialize_database(db_path: &Path, table_schema: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute(table_schema, [])?;
    Ok(conn)
}

/// Sets up the keyboard logging database.
pub fn setup_keyboard_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    // TODO: Change to time stince last epoch instead of timestamp
    ensure_directory(output_dir).expect("Failed to create keyboard output directory");
    let db_path = output_dir.join("keyboard_logs.db");
    initialize_database(
        &db_path,
        "CREATE TABLE IF NOT EXISTS key_events (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME NOT NULL,
            key_code INTEGER NOT NULL,
            action TEXT NOT NULL
        )",
    )
}

/// Sets up the mouse logging database.
pub fn setup_mouse_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create mouse output directory");
    let db_path = output_dir.join("mouse_logs.db");
    initialize_database(
        &db_path,
        "CREATE TABLE IF NOT EXISTS mouse_events (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME NOT NULL,
            x INTEGER NOT NULL,
            y INTEGER NOT NULL,
            button_state TEXT NOT NULL
        )",
    )
}

pub fn setup_system_performance_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create system performance output directory");
    let db_path = output_dir.join("system_metrics.db");
    initialize_database(
        &db_path,
        "CREATE TABLE IF NOT EXISTS system_metrics (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME NOT NULL,
            cpu_usage REAL NOT NULL,
            memory_used INTEGER NOT NULL,
            disk_used INTEGER NOT NULL,
            network_up INTEGER NOT NULL,
            network_down INTEGER NOT NULL
        )",
    )
}

pub fn setup_weather_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create weather output directory");
    let db_path = output_dir.join("weather.db");
    initialize_database(
        &db_path,
        "CREATE TABLE IF NOT EXISTS weather (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME NOT NULL,
            temperature REAL NOT NULL,
            humidity REAL NOT NULL,
            pressure REAL NOT NULL,
            conditions TEXT NOT NULL
        )",
    )
}

pub fn setup_geo_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create geolocation output directory");
    let db_path = output_dir.join("geolocation.db");
    initialize_database(
        &db_path,
        "CREATE TABLE IF NOT EXISTS location (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            accuracy REAL,
            source TEXT NOT NULL
        )",
    )
}

pub fn setup_microphone_db(_output_dir: &Path) -> rusqlite::Result<Connection> {
    // Not implemented yet, but avoid unreachable code warning
    panic!("Not implemented");
    // The following code is commented out to avoid unreachable code warning
    // ensure_directory(output_dir).expect("Failed to create microphone output directory");
    // 
    // let db_path = output_dir.join("microphone.db");
    // let conn = Connection::open(&db_path)?;
    // 
    // conn.execute(
    //     "CREATE TABLE IF NOT EXISTS microphone (
    //         id INTEGER PRIMARY KEY,
    //         timestamp REAL NOT NULL,
    //         path TEXT NOT NULL
    //     )",
    //     params![],
    // )?;
    // 
    // Ok(conn)
}

pub fn setup_wifi_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create wifi output directory");
    let db_path = output_dir.join("wifi.db");
    initialize_database(
        &db_path,
        "CREATE TABLE IF NOT EXISTS wifi_networks (
            id INTEGER PRIMARY KEY,
            timestamp REAL NOT NULL,
            ssid TEXT NOT NULL,
            bssid TEXT NOT NULL,
            signal_strength INTEGER NOT NULL,
            frequency REAL NOT NULL
        )",
    )
}

pub fn setup_hyprland_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create Hyprland output directory");
    let db_path = output_dir.join("hyprland.db");
    let conn = Connection::open(db_path)?;

    // Create tables for each Hyprland data type
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS monitors (
            timestamp REAL NOT NULL,
            id INTEGER NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            refresh_rate REAL NOT NULL,
            x INTEGER NOT NULL,
            y INTEGER NOT NULL,
            active_workspace_id INTEGER NOT NULL,
            active_workspace_name TEXT NOT NULL,
            scale REAL NOT NULL,
            focused BOOLEAN NOT NULL,
            PRIMARY KEY (timestamp, id)
        );

        CREATE TABLE IF NOT EXISTS workspaces (
            timestamp REAL NOT NULL,
            id INTEGER NOT NULL,
            name TEXT NOT NULL,
            monitor TEXT NOT NULL,
            monitor_id INTEGER,
            windows INTEGER NOT NULL,
            fullscreen BOOLEAN NOT NULL,
            last_window TEXT NOT NULL,
            last_window_title TEXT NOT NULL,
            PRIMARY KEY (timestamp, id)
        );

        CREATE TABLE IF NOT EXISTS activeworkspace (
            timestamp REAL NOT NULL,
            id INTEGER NOT NULL,
            name TEXT NOT NULL,
            monitor TEXT NOT NULL,
            monitor_id INTEGER,
            windows INTEGER NOT NULL,
            fullscreen BOOLEAN NOT NULL,
            last_window TEXT NOT NULL,
            last_window_title TEXT NOT NULL,
            PRIMARY KEY (timestamp, id)
        );

        CREATE TABLE IF NOT EXISTS clients (
            timestamp REAL NOT NULL,
            address TEXT NOT NULL,
            at_x INTEGER NOT NULL,
            at_y INTEGER NOT NULL,
            size_x INTEGER NOT NULL,
            size_y INTEGER NOT NULL,
            workspace_id INTEGER NOT NULL,
            workspace_name TEXT NOT NULL,
            floating BOOLEAN NOT NULL,
            fullscreen_mode TEXT NOT NULL,
            monitor_id INTEGER NOT NULL,
            title TEXT NOT NULL,
            class TEXT NOT NULL,
            pid INTEGER NOT NULL,
            pinned BOOLEAN NOT NULL,
            mapped BOOLEAN NOT NULL,
            focus_history_id INTEGER NOT NULL,
            PRIMARY KEY (timestamp, class, pid, address)
        );


        CREATE TABLE IF NOT EXISTS devices (
            timestamp REAL NOT NULL,
            type TEXT NOT NULL,
            name TEXT NOT NULL,
            address TEXT NOT NULL,
            PRIMARY KEY (timestamp, name)
        );

        CREATE TABLE IF NOT EXISTS cursor_positions (
            timestamp REAL PRIMARY KEY NOT NULL,
            x INTEGER NOT NULL,
            y INTEGER NOT NULL
        );

        "#,
    )?;

    Ok(conn)
}
pub fn setup_screen_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create screen output directory");
    let db_path = output_dir.join("screen.db");
    initialize_database(
        &db_path,
        r#"
        CREATE TABLE IF NOT EXISTS screen (
            timestamp REAL NOT NULL,
            PRIMARY KEY (timestamp)
        );
        "#,
    )
}

pub fn setup_process_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create process output directory");
    let db_path = output_dir.join("processes.db");
    initialize_database(
        &db_path,
        r#"
        CREATE TABLE IF NOT EXISTS processes (
            timestamp REAL NOT NULL,
            pid INTEGER NOT NULL,
            ppid INTEGER NOT NULL,
            name TEXT NOT NULL,
            exe TEXT,
            cmdline TEXT,
            status TEXT NOT NULL,
            cpu_usage REAL,
            memory_usage INTEGER,
            threads INTEGER,
            user TEXT,
            start_time REAL,
            PRIMARY KEY (timestamp, pid)
            );
        "#,
    )
}

pub fn setup_embeddings_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create embeddings output directory");
    let db_path = output_dir.join("embeddings.db");
    initialize_database(
        &db_path,
        r#"
        CREATE TABLE IF NOT EXISTS image_embeddings (
            timestamp REAL NOT NULL,
            embedding FLOAT8[] NOT NULL,
            resource_uri TEXT NOT NULL,
            PRIMARY KEY (timestamp)
        );
        "#,
    )
}

pub fn setup_input_logger_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    let db_path = output_dir.join("input.db");
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS input (
            timestamp REAL,
            device_name TEXT NOT NULL,
            event_type TEXT NOT NULL,
            code INTEGER NOT NULL,
            value TEXT NOT NULL,
            primary key (timestamp, device_name, event_type, code, value)
        );
        CREATE TABLE IF NOT EXISTS key_events (
            timestamp REAL PRIMARY KEY,
            event_type TEXT CHECK(event_type IN ('press', 'release')),
            key TEXT NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS mouse_buttons (
            timestamp REAL PRIMARY KEY,
            code TEXT CHECK(event_type IN ('press', 'release')),
            event_type TEXT CHECK(event_type IN ('press', 'release')),
            button TEXT NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS mouse_movements (
            timestamp REAL PRIMARY KEY,
            x REAL NOT NULL,
            y REAL NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS mouse_wheel (
            timestamp REAL PRIMARY KEY,
            delta_x REAL NOT NULL,
            delta_y REAL NOT NULL
        );
        
        CREATE TABLE IF NOT EXISTS devices (
            timestamp REAL,
            event_type TEXT CHECK(event_type IN ('connected', 'disconnected')),
            device_type TEXT NOT NULL,
            device_id TEXT NOT NULL,
            PRIMARY KEY (timestamp, device_id)
        );
        "#,
    )?;

    Ok(conn)
}

pub fn setup_text_upload_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create text upload output directory");
    let db_path = output_dir.join("text_uploads.db");
    initialize_database(
        &db_path,
        "CREATE TABLE IF NOT EXISTS text_files (
            id INTEGER PRIMARY KEY,
            filename TEXT NOT NULL,
            original_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            stored_path TEXT NOT NULL,
            content_hash TEXT NOT NULL,
            upload_date DATETIME NOT NULL
        )",
    )
}
