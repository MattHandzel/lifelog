// Capture the screen using the program of choice

use crate::config::ScreenConfig;
use crate::setup;
use chrono;
use rusqlite::{params, Connection};
use std::path::Path;
use std::process::Command;
use tokio::time::{sleep, Duration};

pub async fn start_logger(config: &ScreenConfig) {
    // check to see if the directory exists
    let conn = setup::setup_screen_db(Path::new(&config.output_dir))
        .expect("Failed to set up process database");
    loop {
        let now = chrono::Local::now();
        let timestamp =
            now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
        let datetime = now.format(config.timestamp_format.as_str());
        let output_path = format!("{}/{}.png", config.output_dir.to_string_lossy(), datetime);

        Command::new("grim")
            .arg("-t")
            .arg("png")
            .arg(&output_path)
            .status()
            .expect("Failed to execute grim");

        conn.execute("INSERT INTO screen VALUES (?1)", params![timestamp])
            .unwrap();

        sleep(Duration::from_secs_f64(config.interval)).await;
    }
}
