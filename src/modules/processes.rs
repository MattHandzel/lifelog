use crate::config::ProcessesConfig;
use crate::setup;
use chrono::Local;
use rusqlite::params;
use std::path::Path;
use sysinfo::{ProcessExt, System, SystemExt};
use tokio::time::{sleep, Duration};

pub async fn start_logger(config: &ProcessesConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let conn = setup::setup_process_db(Path::new(&config.output_dir))
        .expect("Failed to set up process database");

    let mut sys = System::new_all();

    loop {
        let timestamp = Local::now().timestamp() as f64
            + Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;

        // Update all process information
        sys.refresh_all();

        for (pid, process) in sys.processes() {
            conn.execute(
                "INSERT INTO processes VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12
                )",
                params![
                    timestamp,
                    pid.to_string().parse::<i32>().unwrap_or(0),
                    process.parent().map(|p| p.to_string().parse::<i32>().unwrap_or(0)).unwrap_or(0),
                    process.name(),
                    process.exe().to_str(),
                    process.cmd().join(" "),
                    format!("{:?}", process.status()),
                    process.cpu_usage(),
                    process.memory() as i64,
                    process.run_time() as i32,  // Using run_time as a proxy for thread count
                    process.user_id().map(|uid| uid.to_string()),
                    process.start_time() as f64,
                ],
            )
            .unwrap_or_else(|e| {
                eprintln!("Error inserting process data: {}", e);
                0
            });
        }

        sleep(Duration::from_secs_f64(config.interval)).await;
    }
}
