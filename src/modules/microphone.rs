use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavWriter, WavSpec};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::BufWriter;
use tokio::time::{sleep, Duration};
use chrono;
use crate::config::MicrophoneConfig;

pub async fn start_logger(config: &MicrophoneConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Config is {:?}", config);

    let host = cpal::default_host();
    let device = host.default_input_device()
        .ok_or("Failed to get default input device")?;
    let input_config = device.default_input_config()
        .map_err(|e| format!("Failed to get default input format: {}", e))?;
    let stream_config = input_config.config();

    let spec = WavSpec {
        channels: config.channels as u16,
        sample_rate: config.sample_rate as u32,
        bits_per_sample: config.bits_per_sample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    loop {
        println!("Sample rate is {:?}", input_config.sample_rate().0);
        println!("Channels is {:?}", input_config.channels());
        let timestamp = chrono::Local::now().format(config.timestamp_format.as_str());
        let output_path = format!("{}/{}.wav", config.output_dir.to_str().unwrap(), timestamp);

        let writer = WavWriter::create(&output_path, spec)
            .map_err(|e| format!("Failed to create WAV writer: {}", e))?;
        let writer = Arc::new(Mutex::new(Some(writer)));
        let writer_clone = Arc::clone(&writer);

        let stream = device.build_input_stream(
            &stream_config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Some(writer) = &mut *writer_clone.lock().unwrap() {
                    for &sample in data {
                        let sample = (sample * i16::MAX as f32) as i16;
                        writer.write_sample(sample).unwrap();
                    }
                }
            },
            move |err| {
                eprintln!("An error occurred on the input audio stream: {}", err);
            },
            None,
        ).map_err(|e| format!("Failed to build input stream: {}", e))?;

        stream.play().map_err(|e| format!("Failed to play stream: {}", e))?;

        // Sleep for the duration specified in the config before creating a new file
        sleep(Duration::from_secs(config.chunk_duration_secs)).await;

        // Close the current writer
        let mut writer_guard = writer.lock().unwrap();
        if let Some(writer) = writer_guard.take() {
            drop(writer_guard);  // Release the lock before finalizing
            writer.finalize().map_err(|e| format!("Failed to finalize WAV file: {}", e))?;
        }
    }
}
