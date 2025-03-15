use evdev::{Device, EventType, InputEvent};
use rusqlite::Connection;
use rusqlite::params;
use chrono::Local;
use tokio::time::{sleep, Duration};
use crate::config::Config;
use crate::setup;

pub async fn start_logger(config: &Config) {
    // Open the keyboard device
    let mut device = Device::open("/dev/input/event1").expect("Failed to open keyboard device (do you have access to dialout?)");

    // Set up the database
    let conn = setup::setup_keyboard_db(&config.keyboard.output_dir)
        .expect("Failed to set up keyboard database");

    // Main logging loop
    loop {
        for event in device.fetch_events().unwrap() {
            println!("Keyboard {:?}", event);
            if event.event_type() == EventType::KEY {
                let now = Local::now();
                let timestamp = now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
                let key_code = event.code();
                let action = if event.value() == 1 { "press" } else { "release" };

                // Insert into database
                conn.execute(
                    "INSERT INTO key_events (timestamp, key_code, action) VALUES (?1, ?2, ?3)",
                    params![timestamp, key_code, action],
                ).expect("Failed to insert key event");
            }
        }

        // Sleep for the configured interval
        sleep(Duration::from_millis((config.keyboard.interval * 1000.0) as u64)).await;
    }
}
