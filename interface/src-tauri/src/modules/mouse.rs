#[cfg(target_os = "linux")]
use evdev::{Device, EventType, InputEvent};
use rusqlite::Connection;
use crate::config::MouseConfig;
#[cfg(target_os = "linux")]
use crate::setup;
#[cfg(target_os = "linux")]
use rusqlite::params;
#[cfg(target_os = "linux")]
use chrono::Local;
#[cfg(target_os = "linux")]
use tokio::time::{sleep, Duration};

pub async fn start_logger(config: &MouseConfig) {
    #[cfg(target_os = "linux")]
    {
        // Open the mouse device
        let mut device = Device::open("/dev/input/event2").expect("Failed to open mouse device");
        
        // Set up the database
        let conn = setup::setup_mouse_db(&config.output_dir)
            .expect("Failed to set up mouse database");
        
        // Main logging loop
        loop {
            for event in device.fetch_events().unwrap() {
                if event.event_type() == EventType::RELATIVE || event.event_type() == EventType::ABSOLUTE {
                    let now = Local::now();
                    let timestamp = now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
                    let event_type = format!("{:?}", event.event_type());
                    let code = event.code();
                    let value = event.value();
                    
                    // Insert into database
                    conn.execute(
                        "INSERT INTO mouse_events (timestamp, event_type, code, value) VALUES (?1, ?2, ?3, ?4)",
                        params![timestamp, event_type, code, value],
                    ).expect("Failed to insert mouse event");
                }
            }
            
            // Sleep for the configured interval
            sleep(Duration::from_millis((config.interval * 1000.0) as u64)).await;
        }
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        println!("Mouse logging is only available on Linux");
    }
}
