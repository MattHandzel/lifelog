use crate::config::Config;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use sysinfo::{ProcessExt, System, SystemExt};
use std::error::Error;

// Ensures the directory exists, creating it if necessary.
pub fn ensure_directory(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn initialize_project(config: &Config) -> std::io::Result<()> {
    ensure_directory(Path::new(&config.screen.output_dir))?;
    ensure_directory(Path::new(&config.system_performance.output_dir))?;
    ensure_directory(Path::new(&config.ambient.output_dir))?;
    ensure_directory(Path::new(&config.weather.output_dir))?;
    ensure_directory(Path::new(&config.audio.output_dir))?;
    ensure_directory(Path::new(&config.geolocation.output_dir))?;
    ensure_directory(Path::new(&config.wifi.output_dir))?;
    ensure_directory(Path::new(&config.microphone.output_dir))?;
    ensure_directory(Path::new(&config.text_upload.output_dir))?;

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

pub fn setup_screen_db(dir: &Path) -> Result<Connection, Box<dyn Error>> {
    let db_path = dir.join("screen.db");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS screen (
            timestamp REAL PRIMARY KEY
        )",
        [],
    )?;

    // Debug: print the first few entries in the database
    {
        let mut stmt = conn.prepare("SELECT timestamp FROM screen ORDER BY timestamp DESC LIMIT 5")?;
        let rows = stmt.query_map([], |row| {
            let timestamp: f64 = row.get(0)?;
            Ok(timestamp)
        })?;

        println!("Recent screen database entries:");
        for (i, row) in rows.enumerate() {
            if let Ok(timestamp) = row {
                println!("  Entry {}: {}", i+1, timestamp);
            }
        }
        // stmt is dropped here at the end of this block
    }

    Ok(conn)
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

pub fn setup_weather_db(output_dir: &Path) -> Result<Connection, rusqlite::Error> {
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("Failed to create weather output directory: {}", e);
        return Err(rusqlite::Error::InvalidPath(output_dir.to_path_buf()));
    }

    let db_path = output_dir.join("weather.db");
    let conn = Connection::open(&db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS weather (
            timestamp TEXT NOT NULL,
            temperature REAL,
            humidity REAL,
            pressure REAL,
            wind_speed REAL,
            wind_direction TEXT,
            description TEXT
        )",
        [],
    )?;

    Ok(conn)
}

pub fn setup_text_upload_db(output_dir: &Path) -> rusqlite::Result<Connection> {
    ensure_directory(output_dir).expect("Failed to create text upload output directory");
    let db_path = output_dir.join("text_uploads.db");
    initialize_database(
        &db_path,
        r#"
        CREATE TABLE IF NOT EXISTS text_uploads (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp REAL NOT NULL,
            filename TEXT NOT NULL,
            original_path TEXT NOT NULL,
            file_type TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            stored_path TEXT NOT NULL,
            content_hash TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_text_uploads_timestamp ON text_uploads(timestamp);
        CREATE INDEX IF NOT EXISTS idx_text_uploads_filename ON text_uploads(filename);
        CREATE INDEX IF NOT EXISTS idx_text_uploads_file_type ON text_uploads(file_type);
        "#,
    )
}
