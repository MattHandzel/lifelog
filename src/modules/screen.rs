// Capture the screen using the program of choice

use std::process::Command;
use tokio::time::{sleep, Duration};
use chrono;
use crate::config::Config;

pub async fn capture_screenshots(config: &Config) {

    // check to see if the directory exists    
    loop {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S.%3f %Z");
        let output_path = format!("{}/{}.png", config.screen.output_dir, timestamp);
        
        Command::new("grim")
            .arg("-t")
            .arg("png")
            .arg(&output_path)
            .status()
            .expect("Failed to execute grim");
        
        sleep(Duration::from_millis((config.screen.interval * 1000.0) as u64)).await;
    }
}
