use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use chrono::Utc;
use config::MicrophoneConfig;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound::{WavSpec, WavWriter};
use lifelog_core::{LifelogError, Uuid};
use lifelog_types::{to_pb_ts, AudioFrame, RecordType};
use prost::Message;
use std::io::Cursor;
use std::io::{Seek, SeekFrom, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Clone)]
struct SharedCursor {
    inner: Arc<Mutex<Cursor<Vec<u8>>>>,
}

impl Write for SharedCursor {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "cursor mutex poisoned"))?;
        guard.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "cursor mutex poisoned"))?;
        guard.flush()
    }
}

impl Seek for SharedCursor {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "cursor mutex poisoned"))?;
        guard.seek(pos)
    }
}

#[derive(Debug, Clone)]
pub struct AudioDataSource {
    config: MicrophoneConfig,
    pub buffer: Arc<DiskBuffer>,
}

impl AudioDataSource {
    pub fn new(config: MicrophoneConfig) -> Result<Self, LifelogError> {
        let buffer_path = std::path::Path::new(&config.output_dir).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(Self {
            config,
            buffer: Arc::new(buffer),
        })
    }
}

fn record_wav_chunk_blocking(cfg: &MicrophoneConfig) -> Result<(Vec<u8>, u32, u32), LifelogError> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| LifelogError::Validation {
            field: "microphone".to_string(),
            reason: "no default input device available".to_string(),
        })?;

    let supported = device.default_input_config().map_err(|e| {
        LifelogError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;
    let sample_format = supported.sample_format();

    // Use the device defaults for sample rate/channels so `build_input_stream` is likely to succeed.
    // Persist the chosen parameters in the frame for accurate replay/decoding.
    let sample_rate = supported.sample_rate().0;
    let channels = supported.channels();

    let stream_config = cpal::StreamConfig {
        channels,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    let spec = WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let backing = Arc::new(Mutex::new(Cursor::new(Vec::new())));
    let shared = SharedCursor {
        inner: Arc::clone(&backing),
    };

    let writer = WavWriter::new(shared, spec).map_err(|e| {
        LifelogError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to create wav writer: {e}"),
        ))
    })?;
    let writer = Arc::new(Mutex::new(Some(writer)));

    let err_writer = Arc::clone(&writer);
    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            let writer = Arc::clone(&writer);
            device.build_input_stream(
                &stream_config,
                move |data: &[f32], _info| {
                    let Ok(mut guard) = writer.lock() else {
                        return;
                    };
                    let Some(w) = guard.as_mut() else {
                        return;
                    };
                    for &s in data {
                        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
                        let _ = w.write_sample(v);
                    }
                },
                move |err| {
                    tracing::error!(error = %err, "Audio input stream error (f32)");
                    let Ok(mut guard) = err_writer.lock() else {
                        return;
                    };
                    *guard = None;
                },
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let writer = Arc::clone(&writer);
            device.build_input_stream(
                &stream_config,
                move |data: &[i16], _info| {
                    let Ok(mut guard) = writer.lock() else {
                        return;
                    };
                    let Some(w) = guard.as_mut() else {
                        return;
                    };
                    for &s in data {
                        let _ = w.write_sample(s);
                    }
                },
                move |err| {
                    tracing::error!(error = %err, "Audio input stream error (i16)");
                    let Ok(mut guard) = err_writer.lock() else {
                        return;
                    };
                    *guard = None;
                },
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let writer = Arc::clone(&writer);
            device.build_input_stream(
                &stream_config,
                move |data: &[u16], _info| {
                    let Ok(mut guard) = writer.lock() else {
                        return;
                    };
                    let Some(w) = guard.as_mut() else {
                        return;
                    };
                    for &s in data {
                        let v = (s as i32 - i16::MAX as i32) as i16;
                        let _ = w.write_sample(v);
                    }
                },
                move |err| {
                    tracing::error!(error = %err, "Audio input stream error (u16)");
                    let Ok(mut guard) = err_writer.lock() else {
                        return;
                    };
                    *guard = None;
                },
                None,
            )
        }
        other => {
            return Err(LifelogError::Validation {
                field: "microphone".to_string(),
                reason: format!("unsupported sample format: {other:?}"),
            });
        }
    }
    .map_err(|e| {
        LifelogError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        ))
    })?;

    stream.play().map_err(|e| {
        LifelogError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to start audio stream: {e}"),
        ))
    })?;

    // Block for the chunk duration, then drop the stream (stops callbacks).
    let chunk_secs = cfg.chunk_duration_secs.max(1);
    std::thread::sleep(std::time::Duration::from_secs(chunk_secs));
    drop(stream);

    let mut guard = writer.lock().map_err(|_| {
        LifelogError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "wav writer mutex poisoned",
        ))
    })?;
    let Some(w) = guard.take() else {
        return Err(LifelogError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "audio writer became unavailable due to stream error",
        )));
    };

    w.finalize().map_err(|e| {
        LifelogError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("failed to finalize wav: {e}"),
        ))
    })?;

    let bytes = backing
        .lock()
        .map_err(|_| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cursor mutex poisoned",
            ))
        })?
        .get_ref()
        .clone();

    Ok((bytes, sample_rate, channels as u32))
}

#[async_trait]
impl DataSource for AudioDataSource {
    type Config = MicrophoneConfig;

    fn new(config: MicrophoneConfig) -> Result<Self, LifelogError> {
        AudioDataSource::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(AudioBufferedSource {
            stream_id: "audio".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            tracing::warn!("AudioDataSource: Start called but task is already running.");
            return Err(LifelogError::AlreadyRunning);
        }

        RUNNING.store(true, Ordering::SeqCst);
        let source_clone = self.clone();
        let _join_handle = tokio::spawn(async move { source_clone.run().await });

        Ok(DataSourceHandle {
            join: tokio::spawn(async { Ok(()) }),
        })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        RUNNING.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        while RUNNING.load(Ordering::SeqCst) {
            let cfg = self.config.clone();
            let started_at = Utc::now();

            let (wav_bytes, sample_rate, channels) =
                tokio::task::spawn_blocking(move || record_wav_chunk_blocking(&cfg))
                    .await
                    .map_err(|e| {
                        LifelogError::Io(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("audio worker join error: {e}"),
                        ))
                    })??;

            let chunk_secs = self.config.chunk_duration_secs.max(1) as f32;
            let t0 = to_pb_ts(started_at);
            let t1 = to_pb_ts(
                started_at + chrono::Duration::seconds(self.config.chunk_duration_secs as i64),
            );

            let frame = AudioFrame {
                uuid: Uuid::new_v4().to_string(),
                timestamp: t0,
                audio_bytes: wav_bytes,
                codec: "wav".to_string(),
                sample_rate,
                channels,
                duration_secs: chunk_secs,
                t_device: t0,
                t_canonical: t0,
                t_end: t1,
                record_type: RecordType::Interval as i32,
                ..Default::default()
            };

            let mut buf = Vec::new();
            if let Err(e) = frame.encode(&mut buf) {
                tracing::error!(error = %e, "Failed to encode AudioFrame");
            } else if let Err(e) = self.buffer.append(&buf).await {
                tracing::error!(error = %e, "Failed to append AudioFrame to buffer");
            } else {
                tracing::debug!("Stored audio frame in WAL");
            }

            let capture_interval = self
                .config
                .capture_interval_secs
                .max(self.config.chunk_duration_secs)
                .max(1);
            let remaining = capture_interval.saturating_sub(self.config.chunk_duration_secs);
            if remaining > 0 {
                sleep(Duration::from_secs(remaining)).await;
            }
        }

        Ok(())
    }

    fn is_running(&self) -> bool {
        RUNNING.load(Ordering::SeqCst)
    }

    fn get_config(&self) -> Self::Config {
        self.config.clone()
    }
}

pub struct AudioBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for AudioBufferedSource {
    fn stream_id(&self) -> String {
        self.stream_id.clone()
    }

    async fn peek_upload_batch(
        &self,
        max_items: usize,
    ) -> Result<(u64, Vec<Vec<u8>>), LifelogError> {
        let (next_offset, raws) = self.buffer.peek_chunk(max_items).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        Ok((next_offset, raws))
    }

    async fn commit_upload(&self, offset: u64) -> Result<(), LifelogError> {
        self.buffer.commit_offset(offset).await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })
    }
}
