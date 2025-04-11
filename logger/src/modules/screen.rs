use chrono;
use config::ScreenConfig;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{sleep, Duration};
use surrealdb::Surreal;
use surrealdb::sql::{Object, Value};
use surrealdb::Connection;
use serde::{Deserialize, Serialize};
use config::ScreenRecord;
use config::ScreenLog;

static RUNNING: AtomicBool = AtomicBool::new(false);

pub async fn start_logger<C>(config: &ScreenConfig,  db: &Surreal<C>) -> surrealdb::Result<()> where
C: Connection, {
    // Check if already running
    if RUNNING.swap(true, Ordering::SeqCst) {
        return Ok(());
    }
    loop {
        // Check if we should stop
        if !RUNNING.load(Ordering::SeqCst) {
            break Ok (());
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

        let _: Vec<ScreenRecord> = db.upsert("screen").content(ScreenLog {
            datetime: timestamp,
            path: Value::from(output_path).to_string(),
        })
        .await?;

        // // EXAMPLE simple query
        // let records: Vec<Record> = db.select("screen").await?;
        // for record in records {
        //     println!("Record ID: {} datetime: {} path: {}", record.id, record.datetime, record.path);
        // }
        // println!("");

        sleep(Duration::from_secs_f64(config.interval)).await;
    }
}

pub fn stop_logger() {
    RUNNING.store(false, Ordering::SeqCst);
}
