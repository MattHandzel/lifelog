use std::path::{Path, PathBuf};
use rusqlite::Connection;
use std::fs;

/// Ensures the directory exists, creating it if necessary.
pub fn ensure_directory(path: &Path) -> std::io::Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn initialize_project() -> Result<(), Box<dyn std::error::Error> > {
    // TODO: Check to see if all of these things exist or not
    let output_dir = Path::new("output");
    ensure_directory(output_dir)?;
    let keyboard_db = setup_keyboard_db(output_dir)?;
    let mouse_db = setup_mouse_db(output_dir)?;
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
            timestamp DOUBLE NOT NULL,
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
            timestamp DOUBLE NOT NULL,
            x INTEGER NOT NULL,
            y INTEGER NOT NULL,
            button_state TEXT NOT NULL
        )",
    )
}

