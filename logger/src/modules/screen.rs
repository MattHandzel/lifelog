use crate::setup;
use chrono;
use config::ScreenConfig;
use rusqlite::params;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};
use super::logger::LoggerHandle;

static RUNNING: AtomicBool = AtomicBool::new(false);

impl LoggerHandle {
    /// abort the task
    pub fn abort(self) {
        self.join.abort();
    }

    /// await its completion
    pub async fn await_finish(self) {
        let _ = self.join.await;
    }
}

pub fn start_logger(config: &ScreenConfig) -> LoggerHandle {
    let config = config.clone();
    
    let join = tokio::spawn(async move {
        // once-per-process guard
        if RUNNING.swap(true, Ordering::SeqCst) {
            return;
        }

        // Set up the database
        let conn = setup::setup_screen_db(Path::new(&config.output_dir))
            .expect("Failed to set up screen database");

        // main loop
        loop {
            if !RUNNING.load(Ordering::SeqCst) {
                break;
            }

            let now = chrono::Local::now();
            let timestamp =
                now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
            let datetime = now.format(&config.timestamp_format);
            let output_path = format!("{}/{}.png", config.output_dir.display(), datetime);

            // screenshot command per‐OS
            #[cfg(target_os = "macos")]
            {
                std::process::Command::new("screencapture")
                    .arg("-x")
                    .arg("-t")
                    .arg("png")
                    .arg(&output_path)
                    .status()
                    .expect("Failed to execute screenshot");
            }
            #[cfg(not(target_os = "macos"))]
            {
                let cmd = if cfg!(target_os = "linux") { "grim" } else { "screenshot.exe" };
                std::process::Command::new(cmd)
                    .arg("-t")
                    .arg("png")
                    .arg(&output_path)
                    .status()
                    .expect("Failed to execute screenshot");
            }

            conn.execute("INSERT INTO screen VALUES (?1)", params![timestamp])
                .unwrap();

            sleep(Duration::from_secs_f64(config.interval)).await;
        }
    });

    LoggerHandle { join }
}

pub fn stop_logger() {
    RUNNING.store(false, Ordering::SeqCst);
}
