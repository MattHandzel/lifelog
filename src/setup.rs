use std::path::{Path, PathBuf};
use rusqlite::Connection;
use crate::config::Config;
use std::fs;

/// Ensures the directory exists, creating it if necessary.
pub fn ensure_directory(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}


pub fn initialize_project(config: &Config) -> Result<(), Box<dyn std::error::Error> > {
    // TODO: Check to see if all of these things exist or not
    
    
    ensure_directory(Path::new(&config.keyboard.output_dir)).expect("Failed to create keyboard output directory");
    ensure_directory(Path::new(&config.screen.output_dir)).expect("Failed to create keyboard output directory");
    ensure_directory(Path::new(&config.mouse.output_dir)).expect("Failed to create keyboard output directory");
    ensure_directory(Path::new(&config.system_performance.output_dir))?;
    ensure_directory(Path::new(&config.ambient.output_dir))?;
    ensure_directory(Path::new(&config.weather.output_dir))?;
    ensure_directory(Path::new(&config.audio.output_dir))?;
    ensure_directory(Path::new(&config.geolocation.output_dir))?;
    ensure_directory(Path::new(&config.wifi.output_dir))?;

    ensure_directory(Path::new(&config.camera.output_dir))?;


    //let keyboard_db = setup_keyboard_db(output_dir)?;
    //let mouse_db = setup_mouse_db(output_dir)?;
    Ok(())
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
    ensure_directory(output_dir)?;
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
    ensure_directory(output_dir)?;
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
    ensure_directory(output_dir)?;
    let db_path = output_dir.join("geolocation.db");
    initialize_database(
        &db_path,
        "CREATE TABLE IF NOT EXISTS locations (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME NOT NULL,
            latitude REAL NOT NULL,
            longitude REAL NOT NULL,
            accuracy REAL,
            source TEXT NOT NULL
        )",
    )
}

pub fn setup_wifi_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir)?;
    let db_path = output_dir.join("wifi.db");
    initialize_database(
        &db_path,
        "CREATE TABLE IF NOT EXISTS wifi_networks (
            id INTEGER PRIMARY KEY,
            timestamp DATETIME NOT NULL,
            ssid TEXT NOT NULL,
            bssid TEXT NOT NULL,
            signal_strength INTEGER NOT NULL,
            frequency REAL NOT NULL
        )",
    )
}
