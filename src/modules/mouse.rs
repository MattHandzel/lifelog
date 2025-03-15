use evdev::{Device, EventType, InputEvent};
use rusqlite::Connection;
use rusqlite::params;
use chrono::Local;
use tokio::time::{sleep, Duration};
use crate::config::Config;
use crate::setup;

pub async fn start_logger(config: &Config) {
    // Open the mouse device
    let mut device = Device::open("/dev/input/mouse1").expect("Failed to open mouse device");

    // Set up the database
    let conn = setup::setup_mouse_db(&config.mouse.output_dir)
        .expect("Failed to set up mouse database");

    // Main logging loop
    let mut x = 0;
    let mut y = 0;
    let mut button_state = String::new();

    loop {
        for event in device.fetch_events().unwrap() {
            println!("Mouse: {:?}", event);
            match event.event_type() {
                EventType::RELATIVE => {
                    match event.code() {
                        0 => x += event.value(),  // REL_X
                        1 => y += event.value(),  // REL_Y
                        _ => {}
                    }
                }
                EventType::KEY => {
                    button_state = match event.code() {
                        272 => "left".to_string(),    // BTN_LEFT
                        273 => "right".to_string(),   // BTN_RIGHT
                        274 => "middle".to_string(),   // BTN_MIDDLE
                        _ => "unknown".to_string(),
                    };
                }
                _ => {}
            }

            // Log the event
            let now = Local::now();
            let timestamp = now.timestamp() as f64 + now.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;
            conn.execute(
                "INSERT INTO mouse_events (timestamp, x, y, button_state) VALUES (?1, ?2, ?3, ?4)",
                params![timestamp, x, y, button_state],
            ).expect("Failed to insert mouse event");
        }

        // Sleep for the configured interval
        sleep(Duration::from_millis((config.keyboard.interval * 1000.0) as u64)).await;
    }
}
