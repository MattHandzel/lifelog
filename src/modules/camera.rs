use rscam::{Camera, Config, Frame};
use std::path::Path;
use tokio::time::{sleep, Duration};
use chrono::Local;
use crate::config::CameraConfig;
use std::fs;

pub async fn start_logger(config: &CameraConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Create output directory if it doesn't exist
    fs::create_dir_all(&config.output_dir)?;

    // Initialize camera
    let mut camera = Camera::new(&config.device)?;
    
    let camera_config = Config {
        interval: (1, config.fps),  // Frame interval
        format: b"MJPG",           // Motion-JPEG format
        width: config.resolution.0,
        height: config.resolution.1,
        ..Default::default()
    };

    camera.start(&camera_config)?;

    loop {
        let frame = camera.capture()?;
        let timestamp = Local::now().format(&config.timestamp_format);
        let output_path = Path::new(&config.output_dir)
            .join(format!("{}.jpg", timestamp));
        
        save_frame(&frame, &output_path)?;

        sleep(Duration::from_secs_f64(config.interval)).await;
    }
}

fn save_frame(frame: &Frame, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(path, &frame[..])?;
    Ok(())
}
