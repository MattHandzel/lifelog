use chrono::Local;
use config::CameraConfig;
#[cfg(target_os = "linux")]
use rscam::{Camera, Config, Frame};
use std::fs;
#[cfg(target_os = "macos")]
use std::io::ErrorKind;
use std::io::Result;
use std::path::Path;
#[cfg(target_os = "macos")]
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
#[cfg(target_os = "macos")]
use tempfile::NamedTempFile;
use tokio::time::{sleep, Duration};

pub struct CameraFrame {
    pub timestamp: f64,
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub async fn capture_frame(_config: &CameraConfig) -> Result<CameraFrame> {
    #[cfg(target_os = "linux")]
    {
        let mut camera = Camera::new("/dev/video0")?;
        camera
            .start(&Config {
                interval: (1, 30), // 30 fps
                resolution: (640, 480),
                format: b"MJPG",
                ..Default::default()
            })
            .unwrap();

        let frame = camera.capture()?;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        Ok(CameraFrame {
            timestamp,
            data: frame.to_vec(),
            width: 640,
            height: 480,
        })
    }

    #[cfg(target_os = "macos")]
    {
        println!("Attempting to capture frame with imagesnap...");

        // First check if imagesnap is installed
        let check = Command::new("which").arg("imagesnap").output();

        if let Err(e) = check {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                format!(
                    "Failed to check for imagesnap: {}. Make sure it's installed.",
                    e
                ),
            ));
        } else if !check.unwrap().status.success() {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                "imagesnap utility not found. Install with 'brew install imagesnap'",
            ));
        }

        // Check if cameras are available
        let devices = Command::new("imagesnap").arg("-l").output();

        match devices {
            Ok(output) => {
                let devices_str = String::from_utf8_lossy(&output.stdout);
                if !devices_str.contains("Video Devices:") || devices_str.lines().count() <= 1 {
                    return Err(std::io::Error::new(
                        ErrorKind::NotFound,
                        "No cameras were detected by imagesnap. Check camera connections and permissions."
                    ));
                }
                println!("Available cameras: {}", devices_str.trim());
            }
            Err(e) => {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    format!("Failed to list cameras: {}", e),
                ));
            }
        }

        // Get the timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        // Create a temporary file to store the captured image
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_string_lossy().to_string();

        println!("Capturing image to temp file: {}", temp_path);

        // Use imagesnap to capture a frame from the default camera
        let output = Command::new("imagesnap")
            .arg("-w") // Warm-up period of 0.5s for better exposure
            .arg("0.5")
            .arg(&temp_path)
            .output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let stdout = String::from_utf8_lossy(&output.stdout);

                    // Check for common permission errors
                    if stderr.contains("permission")
                        || stderr.contains("denied")
                        || stdout.contains("permission")
                        || stdout.contains("denied")
                    {
                        return Err(std::io::Error::new(
                            ErrorKind::PermissionDenied,
                            "Camera access permission denied. Please grant camera access in System Settings > Privacy & Security > Camera."
                        ));
                    }

                    return Err(std::io::Error::new(
                        ErrorKind::Other,
                        format!("Failed to capture image: {}. Output: {}", stderr, stdout),
                    ));
                }

                // Check if the file was created with content
                let metadata = fs::metadata(&temp_path)?;
                if metadata.len() == 0 {
                    return Err(std::io::Error::new(
                        ErrorKind::Other,
                        "Captured image file is empty. Camera may not be working properly.",
                    ));
                }

                println!("Successfully captured image: {} bytes", metadata.len());

                // Read the captured image
                let image_data = fs::read(&temp_path)?;

                // TODO: Get actual dimensions from image
                // For now, use standard HD resolution
                Ok(CameraFrame {
                    timestamp,
                    data: image_data,
                    width: 1280,
                    height: 720,
                })
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    Err(std::io::Error::new(
                        ErrorKind::NotFound,
                        "imagesnap utility not found. Install with 'brew install imagesnap'",
                    ))
                } else {
                    Err(std::io::Error::new(
                        e.kind(),
                        format!("Failed to execute imagesnap: {}", e),
                    ))
                }
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        // Other platforms - return a dummy error
        use std::io::{Error, ErrorKind};
        Err(Error::new(
            ErrorKind::Unsupported,
            "Camera capture is only supported on Linux and macOS",
        ))
    }
}

pub async fn start_camera_logger(config: &CameraConfig) -> Result<()> {
    if !config.enabled {
        println!("Camera logger not started because it's disabled in config");
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        // Linux-specific implementation
        println!("Starting camera logger (Linux)");

        // Clone the config and spawn the Linux-specific logger
        let config_clone = config.clone();
        tokio::spawn(async move {
            start_logger(&config_clone).await;
        });
    }

    #[cfg(target_os = "macos")]
    {
        // MacOS implementation
        println!("Starting camera logger (macOS)");

        // Create output directory if it doesn't exist
        fs::create_dir_all(&config.output_dir)?;

        // Clone the config and spawn a background task for macOS logger
        let config_clone = config.clone();
        tokio::spawn(async move {
            start_logger(&config_clone).await;
        });
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        println!("Camera logging is not supported on this platform");
    }

    Ok(())
}

#[cfg(target_os = "linux")]
pub async fn start_logger(config: &CameraConfig) {
    // Create output directory if it doesn't exist
    fs::create_dir_all(&config.output_dir).expect("Failed to create camera output directory");

    // Initialize camera
    let mut camera = Camera::new(&config.device).expect("Failed to open camera device");

    let camera_config = Config {
        interval: (1, config.fps), // Frame interval
        format: b"MJPG",           // Motion-JPEG format
        resolution: config.resolution,
        ..Default::default()
    };

    camera
        .start(&camera_config)
        .expect("Failed to start camera");

    loop {
        let frame = camera.capture().expect("Failed to capture frame");
        let timestamp = Local::now().format(&config.timestamp_format.as_str());
        let output_path = Path::new(&config.output_dir).join(format!("{}.jpg", timestamp));

        save_frame(&frame, &output_path).expect("Failed to save frame");

        sleep(Duration::from_secs_f64(config.interval)).await;
    }
}

#[cfg(target_os = "macos")]
pub async fn start_logger(config: &CameraConfig) {
    println!("Starting camera logger on macOS using imagesnap");

    // Verify imagesnap is available
    let check_result = Command::new("which").arg("imagesnap").output();

    match check_result {
        Err(_) => {
            eprintln!("ERROR: imagesnap not found! Please install it with: brew install imagesnap");
            eprintln!("Camera logger cannot start without imagesnap. Exiting.");
            return;
        }
        Ok(output) if !output.status.success() => {
            eprintln!("ERROR: imagesnap not found! Please install it with: brew install imagesnap");
            eprintln!("Camera logger cannot start without imagesnap. Exiting.");
            return;
        }
        _ => {
            // Check if we have available cameras
            let devices_result = Command::new("imagesnap").arg("-l").output();

            match devices_result {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    println!("Available cameras: {}", output_str);

                    if !output_str.contains("Video Devices:") {
                        eprintln!("ERROR: No cameras found by imagesnap!");
                        eprintln!("Make sure your camera is connected and permissions are granted");
                        return;
                    }
                }
                Err(e) => {
                    eprintln!("ERROR checking camera devices: {}", e);
                }
            }
        }
    }

    // Ensure output directory exists
    if let Err(e) = fs::create_dir_all(&config.output_dir) {
        eprintln!("ERROR: Failed to create output directory: {}", e);
        return;
    }

    println!(
        "✓ Camera checks passed! Starting capture loop with interval: {} seconds",
        config.interval
    );

    // Main capture loop
    loop {
        let timestamp = Local::now().format(&config.timestamp_format).to_string();
        let output_path = Path::new(&config.output_dir).join(format!("{}.jpg", timestamp));

        let output_path_str = output_path.to_string_lossy().to_string();

        // Capture with better error handling and output
        let capture_start = std::time::Instant::now();
        println!("Capturing frame to: {}", output_path_str);

        // Use imagesnap to capture a frame
        let result = Command::new("imagesnap")
            .arg("-w") // Warm-up period for better exposure
            .arg("0.5")
            .arg(&output_path_str)
            .output(); // Using output() instead of status() to capture stderr

        match result {
            Ok(output) => {
                if output.status.success() {
                    let duration = capture_start.elapsed();
                    println!("✓ Camera capture successful! Took {:?}", duration);

                    // Check if the file was actually created with content
                    match fs::metadata(&output_path) {
                        Ok(meta) if meta.len() > 0 => {
                            println!(
                                "  ✓ Saved image: {} ({} bytes)",
                                output_path_str,
                                meta.len()
                            );
                        }
                        Ok(_) => {
                            eprintln!("  ⚠ Warning: Saved image file is empty!");
                        }
                        Err(e) => {
                            eprintln!("  ⚠ Warning: Failed to verify saved image: {}", e);
                        }
                    }
                } else {
                    eprintln!("❌ Failed to capture camera frame: {}", output.status);
                    if !output.stderr.is_empty() {
                        eprintln!(
                            "  Error output: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    eprintln!("❌ ERROR: imagesnap utility not found. Install with 'brew install imagesnap'");
                    // Only print the error once, then exit the loop
                    break;
                } else {
                    eprintln!("❌ Failed to execute imagesnap: {}", e);
                }
            }
        }

        // Check if logging is still enabled
        if !config.enabled {
            println!("Camera logging disabled in config, stopping logger");
            break;
        }

        // Wait for the configured interval before next capture
        println!("Waiting {} seconds until next capture...", config.interval);
        sleep(Duration::from_secs_f64(config.interval)).await;
    }

    println!("🛑 Camera logger stopped");
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub async fn start_logger(_config: &CameraConfig) {
    println!("Camera logging is not supported on this platform");
}

fn save_frame(frame: &Frame, path: &Path) -> std::io::Result<()> {
    println!("Save frame not implemented for linux");
    Ok(())
}

//#[cfg(target_os = "linux")]
//fn save_frame(frame: &Frame, path: &Path) -> std::io::Result<()> {
//    match std::fs::write(path, &frame[..]) {
//        Ok(_) => {
//            println!("Frame saved to: {}", path.display());
//            Ok(())
//        }
//        Err(e) => {
//            eprintln!("Failed to save frame: {}", e);
//            Err(e)
//        }
//    }
//}
