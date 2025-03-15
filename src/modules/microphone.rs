use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavWriter, WavSpec};
use std::sync::{Arc, Mutex};
use std::fs::File;
use std::io::BufWriter;
use tokio::time::{sleep, Duration};
use chrono;
use crate::config::MicrophoneConfig;

pub async fn start_logger(config: &MicrophoneConfig) {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("Failed to get default input device");
    let config = device.default_input_format().expect("Failed to get default input format");

    let spec = WavSpec {
        channels: config.channels as u16,
        sample_rate: config.sample_rate,
        bits_per_sample: config.bits_per_sample as u16,
        sample_format: hound::SampleFormat::Int,
    };

    loop {
        let timestamp = chrono::Local::now().format(config.timestamp_format);
        let output_path = format!("{}/{}.wav", config.microphone.output_dir, timestamp);

        let writer = WavWriter::new(BufWriter::new(File::create(&output_path).unwrap()), spec).unwrap();
        let writer = Arc::new(Mutex::new(writer));

        let stream = device.build_input_stream(
            &config,
            {
                let writer = Arc::clone(&writer);
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut writer = writer.lock().unwrap();
                    for &sample in data {
                        let sample = (sample * i16::MAX as f32) as i16;
                        writer.write_sample(sample).unwrap();
                    }
                }
            },
            move |err| {
                eprintln!("An error occurred on the input audio stream: {}", err);
            },
        ).unwrap();

        stream.play().unwrap();
    }
}
