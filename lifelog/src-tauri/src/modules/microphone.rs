use crate::config::MicrophoneConfig;
use chrono;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::fs::{self, File};
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use std::sync::atomic::{AtomicBool, Ordering};
use std::process::Command;
use std::path::PathBuf;

// Global flags for controlling recording state
static RECORDING_ENABLED: AtomicBool = AtomicBool::new(false);
static RECORDING_PAUSED: AtomicBool = AtomicBool::new(false);
static AUTO_RECORDING_ENABLED: AtomicBool = AtomicBool::new(false);

// Shared config for updating settings during runtime
static mut CURRENT_CONFIG: Option<Mutex<MicrophoneConfig>> = None;

pub async fn start_logger(config: &MicrophoneConfig) {
    println!("Starting microphone logger with config: {:?}", config);
    
    // Store config for runtime updates
    unsafe {
        CURRENT_CONFIG = Some(Mutex::new(config.clone()));
    }
    
    // Set initial state based on config
    AUTO_RECORDING_ENABLED.store(config.enabled, Ordering::SeqCst);
    RECORDING_ENABLED.store(false, Ordering::SeqCst);
    RECORDING_PAUSED.store(false, Ordering::SeqCst);

    // Ensure output directory exists
    if let Err(e) = fs::create_dir_all(&config.output_dir) {
        eprintln!("Failed to create output directory: {}", e);
    }

    // Start the auto-recording scheduler
    let config_clone = config.clone();
    tokio::spawn(async move {
        println!("Starting auto-recording scheduler");
        
        loop {
            if !AUTO_RECORDING_ENABLED.load(Ordering::SeqCst) {
                // If auto-recording is disabled, just check again after a short delay
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            
            // Get the current capture interval (may have been updated)
            let capture_interval_secs = unsafe {
                match &CURRENT_CONFIG {
                    Some(cfg) => cfg.lock().unwrap().capture_interval_secs,
                    None => config_clone.capture_interval_secs
                }
            };
            
            // If recording is not already in progress, start a new one
            if !RECORDING_ENABLED.load(Ordering::SeqCst) {
                println!("Auto-recording: starting new recording");
                RECORDING_ENABLED.store(true, Ordering::SeqCst);
                RECORDING_PAUSED.store(false, Ordering::SeqCst);
            }
            
            // Wait for the next scheduled recording time
            let interval = if capture_interval_secs > 0 { 
                capture_interval_secs 
            } else { 
                300 // Default to 5 minutes
            };
            println!("Auto-recording scheduler waiting for {} seconds", interval);
            sleep(Duration::from_secs(interval)).await;
        }
    });

    // Main recording loop
    #[cfg(not(target_os = "macos"))]
    cross_platform_recording_loop(config).await;
    
    #[cfg(target_os = "macos")]
    macos_recording_loop(config).await;
}

#[cfg(not(target_os = "macos"))]
async fn cross_platform_recording_loop(config: &MicrophoneConfig) {
    let host = cpal::default_host();
    let device = match host.default_input_device() {
        Some(d) => d,
        None => {
            eprintln!("No input device available!");
            return;
        }
    };
    
    let input_config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to get default input config: {}", e);
            return;
        }
    };
    
    let stream_config = input_config.config();

    let spec = WavSpec {
        channels: config.channels as u16,
        sample_rate: config.sample_rate as u32,
        bits_per_sample: config.bits_per_sample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    println!("Audio settings - Sample rate: {:?}, Channels: {:?}", 
             input_config.sample_rate().0,
             input_config.channels());

    loop {
        // Check if recording is enabled (manual or auto)
        if !RECORDING_ENABLED.load(Ordering::SeqCst) {
            // Sleep briefly and check again
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        // Get current settings (which may have been updated)
        let current_config = unsafe {
            match &CURRENT_CONFIG {
                Some(cfg) => cfg.lock().unwrap().clone(),
                None => config.clone()
            }
        };

        // Create output directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(&current_config.output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            RECORDING_ENABLED.store(false, Ordering::SeqCst);
            continue;
        }

        let timestamp = chrono::Local::now().format(current_config.timestamp_format.as_str());
        let output_path = format!("{}/{}.wav", current_config.output_dir.to_str().unwrap(), timestamp);
        println!("Creating new audio file: {}", output_path);

        let writer = match WavWriter::create(&output_path, spec) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create WAV file: {}", e);
                RECORDING_ENABLED.store(false, Ordering::SeqCst);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        
        let writer = Arc::new(Mutex::new(Some(writer)));

        let err_writer = Arc::clone(&writer);
        let stream = match device.build_input_stream(
            &stream_config,
            {
                let writer = Arc::clone(&writer);
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // Skip recording if paused
                    if RECORDING_PAUSED.load(Ordering::SeqCst) {
                        return;
                    }
                    
                    if let Some(mut guard) = writer.lock().ok() {
                        if let Some(w) = guard.as_mut() {
                            for &sample in data {
                                let sample = (sample * i16::MAX as f32) as i16;
                                if let Err(e) = w.write_sample(sample) {
                                    eprintln!("Error writing sample: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
            },
            move |err| {
                eprintln!("An error occurred on the input audio stream: {}", err);
                // Clean up the writer on error
                if let Ok(mut guard) = err_writer.lock() {
                    *guard = None;
                }
            },
            None,
        ) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to build input stream: {}", e);
                RECORDING_ENABLED.store(false, Ordering::SeqCst);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        if let Err(e) = stream.play() {
            eprintln!("Failed to start audio stream: {}", e);
            RECORDING_ENABLED.store(false, Ordering::SeqCst);
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        println!("Audio recording started");

        // Create a loop to check for pause/stop
        let mut chunk_timer = Duration::from_secs(0);
        let check_interval = Duration::from_millis(100);
        
        // Get the current chunk duration (may have been updated)
        let chunk_duration_secs = unsafe {
            match &CURRENT_CONFIG {
                Some(cfg) => cfg.lock().unwrap().chunk_duration_secs,
                None => config.chunk_duration_secs
            }
        };
        
        // Keep recording for the specified chunk duration or until disabled
        loop {
            sleep(check_interval).await;
            
            // Check if recording has been disabled
            if !RECORDING_ENABLED.load(Ordering::SeqCst) {
                println!("Recording stopped by user");
                stream.pause().unwrap_or_else(|e| eprintln!("Failed to pause stream: {}", e));
                break;
            }
            
            // Only increment timer if not paused
            if !RECORDING_PAUSED.load(Ordering::SeqCst) {
                chunk_timer += check_interval;
            }
            
            // Check if we've reached the chunk duration
            if chunk_timer.as_secs() >= chunk_duration_secs {
                println!("Chunk duration reached, finalizing recording");
                stream.pause().unwrap_or_else(|e| eprintln!("Failed to pause stream: {}", e));
                
                // If auto-recording is not enabled, stop recording altogether
                if !AUTO_RECORDING_ENABLED.load(Ordering::SeqCst) {
                    RECORDING_ENABLED.store(false, Ordering::SeqCst);
                }
                
                break;
            }
        }
        
        // Finalize the WAV file by explicitly taking and dropping the writer
        if let Ok(mut guard) = writer.lock() {
            if let Some(w) = guard.take() {
                if let Err(e) = w.finalize() {
                    eprintln!("Error finalizing WAV file: {}", e);
                }
            }
        }
        
        println!("Recording chunk completed");
    }
}

#[cfg(target_os = "macos")]
async fn macos_recording_loop(config: &MicrophoneConfig) {
    // Check if we have the required command-line tools
    println!("Using macOS-specific recording implementation");
    
    let has_sox = Command::new("which").arg("sox").output().map(|output| {
        let success = output.status.success();
        if success {
            println!("SoX found at: {}", String::from_utf8_lossy(&output.stdout).trim());
        } else {
            eprintln!("SoX not found: {}", String::from_utf8_lossy(&output.stderr));
        }
        success
    }).unwrap_or_else(|e| {
        eprintln!("Failed to check for SoX: {}", e);
        false
    });
    
    if !has_sox {
        eprintln!("SoX not found. Please install SoX with 'brew install sox' to enable audio recording.");
        return;
    }
    
    // Create base output directory at startup
    let base_dir = config.output_dir.clone();
    println!("Base output directory: {:?}", base_dir);
    
    if let Err(e) = fs::create_dir_all(&base_dir) {
        eprintln!("Failed to create output directory: {}", e);
        // Try to create in home directory as fallback
        if let Some(home_dir) = dirs::home_dir() {
            let fallback_dir = home_dir.join("lifelog_audio");
            println!("Attempting to use fallback directory: {:?}", fallback_dir);
            if let Err(e) = fs::create_dir_all(&fallback_dir) {
                eprintln!("Failed to create fallback directory: {}", e);
                return;
            }
            // Use this as our base directory instead
            unsafe {
                if let Some(ref mutex) = CURRENT_CONFIG {
                    if let Ok(mut current) = mutex.lock() {
                        current.output_dir = fallback_dir.clone();
                        println!("Using fallback directory for recordings: {:?}", fallback_dir);
                    }
                }
            }
        } else {
            eprintln!("Could not determine home directory for fallback");
            return;
        }
    }
    
    loop {
        // Check if recording is enabled (manual or auto)
        if !RECORDING_ENABLED.load(Ordering::SeqCst) {
            // Sleep briefly and check again
            sleep(Duration::from_millis(500)).await;
            continue;
        }

        // Get current settings (which may have been updated)
        let current_config = unsafe {
            match &CURRENT_CONFIG {
                Some(cfg) => cfg.lock().unwrap().clone(),
                None => config.clone()
            }
        };

        // Create output directory if it doesn't exist
        let output_dir = current_config.output_dir.clone();
        println!("Using output directory: {:?}", output_dir);
        
        if let Err(e) = fs::create_dir_all(&output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            RECORDING_ENABLED.store(false, Ordering::SeqCst);
            continue;
        }
        
        // Check directory permissions
        match fs::metadata(&output_dir) {
            Ok(metadata) => {
                let permissions = metadata.permissions();
                println!("Directory permissions: {:?}", permissions);
            },
            Err(e) => {
                eprintln!("Failed to get directory metadata: {}", e);
            }
        }

        let timestamp = chrono::Local::now().format(current_config.timestamp_format.as_str());
        let output_path = format!("{}/{}.wav", output_dir.to_str().unwrap_or("."), timestamp);
        println!("Creating new audio file with SoX: {}", output_path);

        // Test writing to directory
        let test_path = format!("{}/test_write.tmp", output_dir.to_str().unwrap_or("."));
        match fs::write(&test_path, b"test") {
            Ok(_) => {
                println!("Successfully wrote test file to directory");
                let _ = fs::remove_file(&test_path);
            },
            Err(e) => {
                eprintln!("Failed to write test file to directory: {}", e);
                RECORDING_ENABLED.store(false, Ordering::SeqCst);
                continue;
            }
        }

        // Get the recording duration
        let chunk_duration_secs = unsafe {
            match &CURRENT_CONFIG {
                Some(cfg) => cfg.lock().unwrap().chunk_duration_secs,
                None => config.chunk_duration_secs
            }
        };
        
        // Use sox for recording on macOS
        // rec -c 1 -r 44100 -b 16 -e signed-integer output.wav trim 0 60
        let mut cmd = Command::new("rec");
        cmd.arg("-c").arg(current_config.channels.to_string())
           .arg("-r").arg(current_config.sample_rate.to_string())
           .arg("-b").arg(current_config.bits_per_sample.to_string())
           .arg("-e").arg("signed-integer")
           .arg(&output_path)
           .arg("trim").arg("0").arg(chunk_duration_secs.to_string());
           
        // Add debug output
        println!("Starting SoX recording command: {:?}", cmd);
        
        // Start the recording process
        let recording_handle = match cmd.spawn() {
            Ok(child) => {
                println!("Successfully started SoX recording with PID: {}", child.id());
                child
            },
            Err(e) => {
                eprintln!("Failed to start recording with SoX: {}", e);
                
                // Try to directly execute sox to see if it's in PATH
                match Command::new("sox").arg("--version").output() {
                    Ok(output) => {
                        println!("SoX version: {}", String::from_utf8_lossy(&output.stdout));
                    },
                    Err(e) => {
                        eprintln!("Failed to get SoX version: {}", e);
                    }
                }
                
                RECORDING_ENABLED.store(false, Ordering::SeqCst);
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        
        // Check SoX process status in a separate task
        let recording_id = recording_handle.id();
        let recording_task = tokio::spawn(async move {
            match recording_handle.wait_with_output() {
                Ok(output) => {
                    if output.status.success() {
                        println!("SoX recording completed successfully");
                    } else {
                        eprintln!("SoX recording failed with exit code: {:?}", output.status.code());
                        if !output.stderr.is_empty() {
                            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Error waiting for SoX process: {}", e);
                }
            }
        });
        
        // Create a loop to check for pause/stop
        let start_time = std::time::Instant::now();
        let check_interval = Duration::from_millis(100);
        
        // Keep checking until duration is reached or recording is stopped
        loop {
            sleep(check_interval).await;
            
            // Check if recording has been disabled or paused
            if !RECORDING_ENABLED.load(Ordering::SeqCst) || RECORDING_PAUSED.load(Ordering::SeqCst) {
                println!("Recording stopped or paused by user");
                // Kill the SoX process
                let _ = Command::new("kill").arg("-INT").arg(recording_id.to_string()).status();
                recording_task.abort();
                break;
            }
            
            // Check if we've reached the chunk duration
            let elapsed = start_time.elapsed();
            if elapsed.as_secs() >= chunk_duration_secs {
                println!("Chunk duration reached for macOS recording: {} seconds", elapsed.as_secs());
                // The SoX command should terminate on its own after the specified duration
                // Just wait for the task to complete
                break;
            }
        }
        
        // Wait a moment to ensure the file is properly closed
        sleep(Duration::from_millis(500)).await;
        
        // Check if the file was created
        if let Ok(metadata) = fs::metadata(&output_path) {
            println!("Recording file created: {} bytes", metadata.len());
        } else {
            eprintln!("Failed to verify recording file was created");
        }
        
        // If auto-recording is disabled, stop recording altogether
        if !AUTO_RECORDING_ENABLED.load(Ordering::SeqCst) {
            RECORDING_ENABLED.store(false, Ordering::SeqCst);
        }
        
        println!("macOS recording chunk completed");
    }
}

// Function to pause the recording
pub fn pause_recording() {
    RECORDING_PAUSED.store(true, Ordering::SeqCst);
    println!("Recording paused");
}

// Function to resume the recording
pub fn resume_recording() {
    RECORDING_PAUSED.store(false, Ordering::SeqCst);
    println!("Recording resumed");
}

// Function to start manual recording
pub fn start_recording() {
    RECORDING_ENABLED.store(true, Ordering::SeqCst);
    RECORDING_PAUSED.store(false, Ordering::SeqCst);
    println!("Recording started");
}

// Function to stop the recording
pub fn stop_recording() {
    RECORDING_ENABLED.store(false, Ordering::SeqCst);
    println!("Recording stopped");
}

// Function to enable auto recording
pub fn enable_auto_recording() {
    AUTO_RECORDING_ENABLED.store(true, Ordering::SeqCst);
    println!("Auto recording enabled");
}

// Function to disable auto recording
pub fn disable_auto_recording() {
    AUTO_RECORDING_ENABLED.store(false, Ordering::SeqCst);
    println!("Auto recording disabled");
}

// Add new functions to get recording status
pub fn is_recording() -> bool {
    RECORDING_ENABLED.load(Ordering::SeqCst)
}

pub fn is_paused() -> bool {
    RECORDING_PAUSED.load(Ordering::SeqCst)
}

pub fn is_auto_recording_enabled() -> bool {
    AUTO_RECORDING_ENABLED.load(Ordering::SeqCst)
}

// Function to update microphone settings at runtime
pub fn update_settings(config: &MicrophoneConfig) {
    unsafe {
        if let Some(ref mutex) = CURRENT_CONFIG {
            if let Ok(mut current) = mutex.lock() {
                *current = config.clone();
                println!("Updated microphone settings: {:?}", config);
            }
        }
    }
}
