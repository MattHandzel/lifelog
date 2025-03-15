// Capture the screen using the program of choice

use std::process::Command;
use tokio::time::{sleep, Duration};
use chrono;
use crate::config::Config;

pub async fn capture_screenshots(config: &Config) {
    let interval = Duration::from_secs(config.screen.interval);

    // check to see if the directory exists    
    loop {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S.%3f %Z");
        let output_path = format!("{}/{}.png", config.screen.dir, timestamp);
        
        Command::new("grim")
            .arg("-t")
            .arg("png")
            .arg(&output_path)
            .status()
            .expect("Failed to execute grim");
        
        sleep(interval).await;
    }
}
