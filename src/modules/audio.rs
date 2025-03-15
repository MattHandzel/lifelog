// src/modules/audio.rs
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use opus::Encoder;

pub struct AudioRecorder {
    encoder: Encoder,
    buffer: Vec<i16>,
}

impl AudioRecorder {
    pub fn new() -> Self {
        let encoder = Encoder::new(48000, opus::Channels::Stereo, opus::Application::Audio).unwrap();
        AudioRecorder {
            encoder,
            buffer: Vec::with_capacity(960), // 20ms @ 48kHz
        }
    }

    pub fn start(&mut self) {
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        let config = device.default_input_config().unwrap();
        
        let stream = device.build_input_stream(
            &config.config(),
            move |data: &[f32], _: &_| {
                // Convert to i16 and encode
                let pcm: Vec<i16> = data.iter()
                    .map(|&x| (x * i16::MAX as f32) as i16)
                    .collect();
                
                self.buffer.extend_from_slice(&pcm);
                if self.buffer.len() >= 960 {
                    let mut output = [0u8; 4000];
                    let len = self.encoder.encode(&self.buffer, &mut output).unwrap();
                    // Write output to file with timestamp
                    self.buffer.clear();
                }
            },
            |err| eprintln!("Audio error: {}", err),
            None
        ).unwrap();
        
        stream.play().unwrap();
        std::thread::park();
    }
}
