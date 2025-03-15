// Capture the screen using the program of choice

use std::process::Command;
use tokio::time::{sleep, Duration};
use crate::config::Config;

pub async fn capture_screenshots(config: &Config) {
    let interval = Duration::from_secs(config.screen_interval);
    
    loop {
        let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S.%3f %Z");
        let output_path = format!("{}/{}.jxl", config.screen_dir, timestamp);
        
        Command::new("grim")
            .arg("-t")
            .arg("jxl")
            .arg(&output_path)
            .status()
            .expect("Failed to execute grim");
        
        sleep(interval).await;
    }
}
