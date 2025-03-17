use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavWriter, WavSpec};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::BufWriter;
use tokio::time::{sleep, Duration};
use chrono;
use crate::config::MicrophoneConfig;

pub async fn start_logger(config: &MicrophoneConfig) {
    println!("Config is {:?}", config);

    let host = cpal::default_host();
    let device = host.default_input_device().expect("Failed to get default input device");
    let input_config = device.default_input_config().expect("Failed to get default input format");
    let stream_config = input_config.config();

    let spec = WavSpec {
        channels: input_config.channels() as u16,
        sample_rate: input_config.sample_rate().0 as u32,
        bits_per_sample: config.bits_per_sample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    loop {
        println!("Sample rate is {:?}", input_config.sample_rate().0);
        println!("Channels is {:?}", input_config.channels());
        let timestamp = chrono::Local::now().format(config.timestamp_format.as_str());
        let output_path = format!("{}/{}.wav", config.output_dir.to_str().unwrap(), timestamp);

        let writer = WavWriter::create(&output_path, spec);
        let writer = Arc::new(Mutex::new(writer));

        let stream = device.build_input_stream(
            &stream_config,
            {
                let writer = Arc::clone(&writer);
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut writer = writer.lock().unwrap();
                    for &sample in data {
                        let sample = (sample * i16::MAX as f32) as i16;
                        writer.as_mut().unwrap().write_sample(sample).unwrap();
                    }
                }
            },
            move |err| {
                eprintln!("An error occurred on the input audio stream: {}", err);
            },
            None, // Added the missing Optional timeout parameter
        ).unwrap();

        stream.play().unwrap();

        // Sleep for the duration specified in the config before creating a new file
        sleep(Duration::from_secs(config.chunk_duration_secs)).await;
    }
}
