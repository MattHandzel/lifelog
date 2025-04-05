use chrono;
use config::ScreenConfig;
use rusqlite::{params, Connection};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};

static RUNNING: AtomicBool = AtomicBool::new(false);

pub async fn start_logger(config: &ScreenConfig) {
    // Check if already running
    if RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    // Set up the database
    let conn = setup::setup_screen_db(Path::new(&config.output_dir))
        .expect("Failed to set up screen database");

    loop {
        // Check if we should stop
        if !RUNNING.load(Ordering::SeqCst) {
            break;
        }

        let now = chrono::Local::now();
        let timestamp =
            now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
        let datetime = now.format(config.timestamp_format.as_str());
        let output_path = format!("{}/{}.png", config.output_dir.to_string_lossy(), datetime);

        #[cfg(target_os = "linux")]
        let command = "grim";

        #[cfg(target_os = "macos")]
        let command = "screencapture";

        #[cfg(target_os = "windows")]
        let command = "screenshot.exe";

        // Create platform-specific command with appropriate arguments
        #[cfg(target_os = "macos")]
        {
            // On macOS, add the silent mode flag
            Command::new(command)
                .arg("-x") // Silent mode, no UI sounds or visual feedback
                .arg("-t")
                .arg("png")
                .arg(&output_path)
                .status()
                .expect("Failed to execute screenshot command");
        }
        #[cfg(not(target_os = "macos"))]
        {
            // Original command for Linux and Windows
            Command::new(command)
                .arg("-t")
                .arg("png")
                .arg(&output_path)
                .status()
                .expect("Failed to execute screenshot command");
        }

        conn.execute("INSERT INTO screen VALUES (?1)", params![timestamp])
            .unwrap();

        sleep(Duration::from_secs_f64(config.interval)).await;
    }
}

pub fn stop_logger() {
    RUNNING.store(false, Ordering::SeqCst);
}
