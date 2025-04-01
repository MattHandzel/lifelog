use rscam::{Camera, Config, Frame};
use std::path::Path;
use tokio::time::{sleep, Duration};
use chrono::Local;
use crate::config::CameraConfig;
use std::fs;

pub async fn start_logger(config: &CameraConfig) {
    // Create output directory if it doesn't exist
    fs::create_dir_all(&config.output_dir).expect("Failed to create camera output directory");

    // Initialize camera
    let mut camera = Camera::new(&config.device).expect("Failed to open camera device");
    
    let camera_config = Config {
        interval: (1, config.fps),  // Frame interval
        format: b"MJPG",           // Motion-JPEG format
        resolution: config.resolution,
        ..Default::default()
    };

    camera.start(&camera_config).expect("Failed to start camera");

    loop {
        let frame = camera.capture().expect("Failed to capture frame");
        let timestamp = Local::now().format(&config.timestamp_format.as_str());
        let output_path = Path::new(&config.output_dir)
            .join(format!("{}.jpg", timestamp));
        
        save_frame(&frame, &output_path).expect("Failed to save frame");

        sleep(Duration::from_secs_f64(config.interval)).await;
    }
}

fn save_frame(frame: &Frame, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::write(path, &frame[..])?;
    Ok(())
}
