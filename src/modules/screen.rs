// Capture the screen using the program of choice

use crate::config::ScreenConfig;
use crate::setup;
use chrono;
use rusqlite::{params, Connection};
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::watch;
use tokio::time::{sleep, Duration};

// Global flag to track if a logger is running
static LOGGER_RUNNING: AtomicBool = AtomicBool::new(false);

pub async fn start_logger(
    config: &ScreenConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // If screenshots are disabled, just return
    if !config.enabled {
        println!("Screenshot capture is disabled");
        return Ok(());
    }

    // Check if there's already a logger running
    if LOGGER_RUNNING.swap(true, Ordering::SeqCst) {
        // Another logger is already running
        println!("Screenshot logger is already running. Stopping the previous one...");
        // We'll rely on the existing logger checking this flag and exiting
        sleep(Duration::from_millis(100)).await;
    }

    // Making sure the output directory exists
    fs::create_dir_all(&config.output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;

    println!("Saving screenshots to: {}", config.output_dir.display());

    // Check to see if the directory exists
    let conn = setup::setup_screen_db(Path::new(&config.output_dir))
        .expect("Failed to set up screen database");

    let interval = config.interval;

    // Clone config values we need in the loop
    let timestamp_format = config.timestamp_format.clone();
    let output_dir = config.output_dir.clone();

    loop {
        // Check if screenshots have been disabled
        if !LOGGER_RUNNING.load(Ordering::SeqCst) {
            println!("Screenshot logger stopping as requested");
            // Reset the flag in case we want to start again
            LOGGER_RUNNING.store(false, Ordering::SeqCst);
            return Ok(());
        }

        let now = chrono::Local::now();
        let timestamp =
            now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;

        // Format the filename - ensure lowercase to avoid case sensitivity issues
        let datetime_str = now.format(timestamp_format.as_str()).to_string();
        let filename = format!("{}.png", datetime_str);

        // Log the exact format for easier debugging
        println!("Timestamp format string: {}", timestamp_format.as_str());
        println!("Timestamp: {}, Formatted filename: {}", timestamp, filename);

        let output_path = format!("{}/{}", output_dir.to_string_lossy(), filename);
        println!("Taking screenshot: {}", output_path);

        // Use screencapture on macOS
        #[cfg(target_os = "macos")]
        let result = Command::new("screencapture")
            .arg("-x")
            .arg("-C")
            .arg(&output_path)
            .output()
            .map_err(|e| format!("Failed to execute screencapture: {}", e))?;

        #[cfg(target_os = "linux")]
        let result = Command::new("grim")
            .arg("-t")
            .arg("png")
            .arg(&output_path)
            .output()
            .map_err(|e| format!("Failed to execute screencapture: {}", e))?;

        if !result.status.success() {
            eprintln!(
                "Screenshot failed: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            continue;
        }

        // Verify the file was created
        if !Path::new(&output_path).exists() {
            eprintln!("Screenshot file was not created: {}", output_path);
            continue;
        }

        conn.execute("INSERT INTO screen VALUES (?1)", params![timestamp])
            .map_err(|e| format!("Failed to insert into database: {}", e))?;

        sleep(Duration::from_secs_f64(interval)).await;
    }
}

// Helper function to stop the logger
pub fn stop_logger() {
    LOGGER_RUNNING.store(false, Ordering::SeqCst);
    println!("Screenshot logger stop requested");
}
