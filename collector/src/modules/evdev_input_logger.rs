use crate::setup;
use chrono::Local;
use config::InputLoggerConfig;
#[cfg(target_os = "linux")]
use evdev::*;
use rusqlite::{params, Connection};
use std::fs;
use std::time::SystemTime;
use tokio::time::{sleep, Duration};

#[cfg(target_os = "linux")]
pub async fn start_logger(config: &InputLoggerConfig) {
    // Set up database once to ensure tables exist
    let _ = setup::setup_input_logger_db(&config.output_dir)
        .expect("Failed to initialize database schema");

    // Iterate through all devices in /dev/input/
    let entries = fs::read_dir("/dev/input/").expect("Failed to read /dev/input directory");

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading directory entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        let path_str = match path.to_str() {
            Some(s) => s,
            None => {
                eprintln!("Invalid path: {:?}", path);
                continue;
            }
        };

        // Try to open each input device
        let mut device = match Device::open(path_str) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to open {}: {} (permissions?)", path_str, e);
                continue;
            }
        };

        // Configure non-blocking mode for async compatibility
        if let Err(e) = device.set_nonblocking(true) {
            eprintln!("Failed to set non-blocking mode for {}: {}", path_str, e);
            continue;
        }

        let device_name = device.name().unwrap_or("unknown").to_string();
        let output_dir = config.output_dir.clone();

        // Spawn a separate async task for each device
        tokio::spawn(async move {
            // Each device gets its own database connection
            let conn = match setup::setup_input_logger_db(&output_dir) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to create database connection: {}", e);
                    return;
                }
            };

            // Device-specific processing loop
            loop {
                let timestamp = Local::now().timestamp() as f64
                    + Local::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0;

                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            let event_timestamp_unwrapped = event
                                .timestamp()
                                .duration_since(SystemTime::UNIX_EPOCH)
                                .unwrap();
                            if event.event_type() == EventType::SYNCHRONIZATION
                                || event.event_type() == EventType::MISC
                            {
                                continue;
                            }

                            if let Err(e) = conn.execute(
                                "INSERT  INTO input (timestamp, device_name, event_type, code, value) VALUES (?1, ?2, ?3, ?4, ?5)",
                                params![
                                    event_timestamp_unwrapped.as_secs() as f64
                                        + event_timestamp_unwrapped.subsec_nanos() as f64 / 1_000_000_000.0,
                                    device_name,
                                    format!("{:?}", event.event_type()),
                                    event.code(),
                                    event.value()
                                ],
                            ) {
                                eprintln!("Database error: {}", e);
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No events available, yield to scheduler
                        sleep(Duration::from_millis(10)).await;
                    }
                    Err(e) => {
                        eprintln!("Critical error reading events: {}", e);
                        break;
                    }
                }
            }
        });
    }

    // Keep the main task alive indefinitely
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}

#[cfg(not(target_os = "linux"))]
pub async fn start_logger(config: &InputLoggerConfig) {
    println!("Evdev input logging is only available on Linux");

    // Set up database to ensure tables exist
    let _ = setup::setup_input_logger_db(&config.output_dir)
        .expect("Failed to initialize database schema");

    // Keep the main task alive indefinitely
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}
