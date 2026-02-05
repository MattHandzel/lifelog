use crate::logger::*;
use async_trait::async_trait;
use chrono;
use config::MicrophoneConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::time::{sleep, Duration as TokioDuration};

#[cfg(target_os = "macos")]
use std::process::Command;

// Global flags for controlling recording state
static RECORDING_ENABLED: AtomicBool = AtomicBool::new(false);
static RECORDING_PAUSED: AtomicBool = AtomicBool::new(false);
static AUTO_RECORDING_ENABLED: AtomicBool = AtomicBool::new(false);

// Shared config for updating settings during runtime
static mut CURRENT_CONFIG: Option<Mutex<MicrophoneConfig>> = None;

pub struct MicrophoneLogger {
    config: MicrophoneConfig,
}

impl MicrophoneLogger {
    pub fn new(config: MicrophoneConfig) -> Result<Self, LoggerError> {
        Ok(MicrophoneLogger { config })
    }

    pub fn setup(&self) -> Result<LoggerHandle, LoggerError> {
        DataLogger::setup(self, self.config.clone())
    }
}

#[async_trait]
impl DataLogger for MicrophoneLogger {
    type Config = MicrophoneConfig;

    fn new(config: MicrophoneConfig) -> Result<Self, LoggerError> {
        MicrophoneLogger::new(config)
    }

    fn setup(&self, config: MicrophoneConfig) -> Result<LoggerHandle, LoggerError> {
        let logger = Self::new(config)?;
        let join = tokio::spawn(async move {

            let task_result = logger.run().await;

            println!("[Task] Background task finished with result: {:?}", task_result);

            task_result
        });

        Ok(LoggerHandle { join })
    }

    async fn run(&self) -> Result<(), LoggerError> {
        let config = self.config.clone();
        println!("Starting microphone logger with config: {:?}", config);

        // Store config for runtime updates
        unsafe {
            CURRENT_CONFIG = Some(Mutex::new(config.clone()));
        }

        // Initialize flags
        AUTO_RECORDING_ENABLED.store(config.enabled, Ordering::SeqCst);
        RECORDING_ENABLED.store(false, Ordering::SeqCst);
        RECORDING_PAUSED.store(false, Ordering::SeqCst);

        // Ensure output directory exists
        if let Err(e) = fs::create_dir_all(&config.output_dir) {
            eprintln!("Failed to create output directory: {}", e);
        }

        // Spawn the auto‑recording scheduler
        {
            let cfg = config.clone();
            tokio::spawn(async move {
                println!("Starting auto‑recording scheduler");
                loop {
                    if !AUTO_RECORDING_ENABLED.load(Ordering::SeqCst) {
                        sleep(TokioDuration::from_secs(1)).await;
                        continue;
                    }
                    let interval_secs = unsafe {
                        CURRENT_CONFIG
                            .as_ref()
                            .map(|m| m.lock().unwrap().capture_interval_secs)
                            .unwrap_or(cfg.capture_interval_secs)
                    }
                    .max(1);

                    if !RECORDING_ENABLED.load(Ordering::SeqCst) {
                        println!("Auto‑recording: starting new recording");
                        RECORDING_ENABLED.store(true, Ordering::SeqCst);
                        RECORDING_PAUSED.store(false, Ordering::SeqCst);
                    }
                    println!("Scheduler waiting {}s", interval_secs);
                    sleep(TokioDuration::from_secs(interval_secs)).await;
                }
            });
        }

        // Kick off the platform‑specific recording loop in its own thread
        #[cfg(not(target_os = "macos"))]
        {
            let rec_cfg = config.clone();
            thread::spawn(move || blocking_cross_platform_recording_loop(rec_cfg));
        }

        #[cfg(target_os = "macos")]
        {
            let rec_cfg = config.clone();
            thread::spawn(move || blocking_macos_recording_loop(rec_cfg));
        }

        Ok(())
    }

    fn stop(&self) {
        RECORDING_ENABLED.store(false, Ordering::SeqCst);
        AUTO_RECORDING_ENABLED.store(false, Ordering::SeqCst);
    }

    async fn log_data(&self) -> Result<(), LoggerError> {
        Ok(())
    }
}

// Blocking, sync recording loops:
#[cfg(not(target_os = "macos"))]
fn blocking_cross_platform_recording_loop(config: MicrophoneConfig) {
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

    println!(
        "Audio settings - Sample rate: {:?}, Channels: {:?}",
        input_config.sample_rate().0,
        input_config.channels()
    );

    loop {
        if !RECORDING_ENABLED.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(500));
            continue;
        }

        let current_config = unsafe {
            CURRENT_CONFIG
                .as_ref()
                .map(|m| m.lock().unwrap().clone())
                .unwrap_or_else(|| config.clone())
        };

        if let Err(e) = fs::create_dir_all(&current_config.output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            RECORDING_ENABLED.store(false, Ordering::SeqCst);
            continue;
        }

        let timestamp = chrono::Local::now().format(current_config.timestamp_format.as_str());
        let output_path = format!(
            "{}/{}.wav",
            current_config.output_dir.to_str().unwrap(),
            timestamp
        );
        println!("Creating new audio file: {}", output_path);

        let writer = match WavWriter::create(&output_path, spec) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("Failed to create WAV file: {}", e);
                RECORDING_ENABLED.store(false, Ordering::SeqCst);
                thread::sleep(Duration::from_secs(1));
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
                    if RECORDING_PAUSED.load(Ordering::SeqCst) {
                        return;
                    }
                    if let Ok(mut guard) = writer.lock() {
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
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        if let Err(e) = stream.play() {
            eprintln!("Failed to start audio stream: {}", e);
            RECORDING_ENABLED.store(false, Ordering::SeqCst);
            thread::sleep(Duration::from_secs(1));
            continue;
        }

        println!("Audio recording started");

        let mut chunk_timer = Duration::from_secs(0);
        let check_interval = Duration::from_millis(100);
        let chunk_duration_secs = unsafe {
            CURRENT_CONFIG
                .as_ref()
                .map(|m| m.lock().unwrap().chunk_duration_secs)
                .unwrap_or(config.chunk_duration_secs)
        };

        loop {
            thread::sleep(check_interval);

            if !RECORDING_ENABLED.load(Ordering::SeqCst) {
                println!("Recording stopped by user");
                stream
                    .pause()
                    .unwrap_or_else(|e| eprintln!("Failed to pause stream: {}", e));
                break;
            }

            if !RECORDING_PAUSED.load(Ordering::SeqCst) {
                chunk_timer += check_interval;
            }

            if chunk_timer.as_secs() >= chunk_duration_secs {
                println!("Chunk duration reached, finalizing recording");
                stream
                    .pause()
                    .unwrap_or_else(|e| eprintln!("Failed to pause stream: {}", e));
                if !AUTO_RECORDING_ENABLED.load(Ordering::SeqCst) {
                    RECORDING_ENABLED.store(false, Ordering::SeqCst);
                }
                break;
            }
        }

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
fn blocking_macos_recording_loop(config: MicrophoneConfig) {
    println!("Using macOS-specific recording implementation");

    let has_sox = Command::new("which")
        .arg("sox")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !has_sox {
        eprintln!(
            "SoX not found. Please install SoX with 'brew install sox' to enable audio recording."
        );
        return;
    }

    let base_dir = config.output_dir.clone();
    if let Err(e) = fs::create_dir_all(&base_dir) {
        eprintln!("Failed to create output directory: {}", e);
        return;
    }

    loop {
        if !RECORDING_ENABLED.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(500));
            continue;
        }

        let current_config = unsafe {
            CURRENT_CONFIG
                .as_ref()
                .map(|m| m.lock().unwrap().clone())
                .unwrap_or_else(|| config.clone())
        };

        if let Err(e) = fs::create_dir_all(&current_config.output_dir) {
            eprintln!("Failed to create output directory: {}", e);
            RECORDING_ENABLED.store(false, Ordering::SeqCst);
            continue;
        }

        let timestamp = chrono::Local::now().format(current_config.timestamp_format.as_str());
        let output_path = format!(
            "{}/{}.wav",
            current_config.output_dir.to_str().unwrap(),
            timestamp
        );
        println!("Creating new audio file with SoX: {}", output_path);

        let mut cmd = Command::new("rec");
        cmd.arg("-c")
            .arg(current_config.channels.to_string())
            .arg("-r")
            .arg(current_config.sample_rate.to_string())
            .arg("-b")
            .arg(current_config.bits_per_sample.to_string())
            .arg("-e")
            .arg("signed-integer")
            .arg(&output_path)
            .arg("trim")
            .arg("0")
            .arg(current_config.chunk_duration_secs.to_string());

        println!("Starting SoX recording command: {:?}", cmd);

        let mut recording = match cmd.spawn() {
            Ok(child) => child,
            Err(e) => {
                eprintln!("Failed to start recording with SoX: {}", e);
                RECORDING_ENABLED.store(false, Ordering::SeqCst);
                thread::sleep(Duration::from_secs(1));
                continue;
            }
        };

        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < config.chunk_duration_secs {
            thread::sleep(Duration::from_millis(100));
            if !RECORDING_ENABLED.load(Ordering::SeqCst) {
                let _ = recording.kill();
                break;
            }
        }

        let _ = recording.wait();
        println!("macOS recording chunk completed");
    }
}

// Control functions
pub fn pause_recording() {
    RECORDING_PAUSED.store(true, Ordering::SeqCst);
    println!("Recording paused");
}
pub fn resume_recording() {
    RECORDING_PAUSED.store(false, Ordering::SeqCst);
    println!("Recording resumed");
}
pub fn start_recording() {
    RECORDING_ENABLED.store(true, Ordering::SeqCst);
    RECORDING_PAUSED.store(false, Ordering::SeqCst);
    println!("Recording started");
}
pub fn stop_recording() {
    RECORDING_ENABLED.store(false, Ordering::SeqCst);
    println!("Recording stopped");
}
pub fn enable_auto_recording() {
    AUTO_RECORDING_ENABLED.store(true, Ordering::SeqCst);
    println!("Auto recording enabled");
}
pub fn disable_auto_recording() {
    AUTO_RECORDING_ENABLED.store(false, Ordering::SeqCst);
    println!("Auto recording disabled");
}

pub fn is_recording() -> bool {
    RECORDING_ENABLED.load(Ordering::SeqCst)
}
pub fn is_paused() -> bool {
    RECORDING_PAUSED.load(Ordering::SeqCst)
}
pub fn is_auto_recording_enabled() -> bool {
    AUTO_RECORDING_ENABLED.load(Ordering::SeqCst)
}

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
