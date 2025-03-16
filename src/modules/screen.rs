// Capture the screen using the program of choice

use std::process::Command;
use tokio::time::{sleep, Duration};
use chrono;
use crate::config::ScreenConfig;

pub async fn start_logger(config: &ScreenConfig) {

    // check to see if the directory exists    
    loop {
        let timestamp = chrono::Local::now().format(config.timestamp_format.as_str());
        let output_path = format!("{}/{}.png", config.output_dir, timestamp);
        
        Command::new("grim")
            .arg("-t")
            .arg("png")
            .arg(&output_path)
            .status()
            .expect("Failed to execute grim");
        
        sleep(Duration::from_millis((config.interval * 1000.0) as u64)).await;
    }
}
