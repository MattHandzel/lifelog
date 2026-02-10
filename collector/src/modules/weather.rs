use crate::data_source::{BufferedSource, DataSource, DataSourceHandle};
use async_trait::async_trait;
use config::WeatherConfig;
use lifelog_core::{LifelogError, Utc, Uuid};
use lifelog_types::{to_pb_ts, WeatherFrame};
use prost::Message;
use reqwest::Client;
use serde_json::Value;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use utils::buffer::DiskBuffer;

static RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
pub struct WeatherDataSource {
    config: WeatherConfig,
    pub buffer: Arc<DiskBuffer>,
}

impl WeatherDataSource {
    pub fn new(config: WeatherConfig) -> Result<Self, LifelogError> {
        let buffer_path = std::path::Path::new(&config.output_dir).join("buffer");
        let buffer = DiskBuffer::new(&buffer_path).map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        Ok(WeatherDataSource {
            config,
            buffer: Arc::new(buffer),
        })
    }
}

// Function to get API key from environment if available
fn get_weather_api_key(config_api_key: &str) -> String {
    env::var("WEATHER_API_KEY").unwrap_or_else(|_| config_api_key.to_string())
}

#[async_trait]
impl DataSource for WeatherDataSource {
    type Config = WeatherConfig;

    fn new(config: WeatherConfig) -> Result<Self, LifelogError> {
        WeatherDataSource::new(config)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        Some(Arc::new(WeatherBufferedSource {
            stream_id: "weather".to_string(),
            buffer: self.buffer.clone(),
        }))
    }

    fn start(&self) -> Result<DataSourceHandle, LifelogError> {
        if RUNNING.load(Ordering::SeqCst) {
            return Err(LifelogError::AlreadyRunning);
        }

        tracing::info!("WeatherDataSource: Starting data source task");
        RUNNING.store(true, Ordering::SeqCst);

        let source_clone = self.clone();

        let _join_handle = tokio::spawn(async move {
            let task_result = source_clone.run().await;
            tracing::info!(result = ?task_result, "WeatherDataSource background task finished");
            task_result
        });

        let new_join_handle = tokio::spawn(async { Ok(()) });
        Ok(DataSourceHandle {
            join: new_join_handle,
        })
    }

    async fn stop(&mut self) -> Result<(), LifelogError> {
        RUNNING.store(false, Ordering::SeqCst);
        Ok(())
    }

    async fn run(&self) -> Result<(), LifelogError> {
        let api_key = get_weather_api_key(&self.config.api_key);

        if api_key.is_empty() {
            tracing::error!("Weather API key is not set!");
            return Err(LifelogError::SourceSetup(
                "weather".to_string(),
                "API key is missing".to_string(),
            ));
        }

        let client = Client::new();
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=metric",
            self.config.latitude, self.config.longitude, api_key
        );

        while RUNNING.load(Ordering::SeqCst) {
            match client.get(&url).send().await {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<Value>().await {
                        if let (Some(main), Some(weather_arr)) =
                            (json["main"].as_object(), json["weather"].as_array())
                        {
                            if let Some(weather) = weather_arr.first().and_then(|w| w.as_object()) {
                                let timestamp = to_pb_ts(Utc::now());
                                let frame = WeatherFrame {
                                    uuid: Uuid::new_v4().to_string(),
                                    timestamp,
                                    temperature: main["temp"].as_f64().unwrap_or(0.0),
                                    humidity: main["humidity"].as_f64().unwrap_or(0.0),
                                    pressure: main["pressure"].as_f64().unwrap_or(0.0),
                                    conditions: weather["main"]
                                        .as_str()
                                        .unwrap_or("Unknown")
                                        .to_string(),
                                    t_device: timestamp,
                                    t_canonical: timestamp,
                                    t_end: timestamp,
                                    ..Default::default()
                                };

                                let mut buf = Vec::new();
                                if let Err(e) = frame.encode(&mut buf) {
                                    tracing::error!("Failed to encode WeatherFrame: {}", e);
                                } else if let Err(e) = self.buffer.append(&buf).await {
                                    tracing::error!(
                                        "Failed to append WeatherFrame to buffer: {}",
                                        e
                                    );
                                } else {
                                    tracing::debug!("Stored weather frame in WAL");
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Weather API request failed: {}", e);
                }
            }
            sleep(Duration::from_secs_f64(self.config.interval)).await;
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

pub struct WeatherBufferedSource {
    stream_id: String,
    buffer: Arc<DiskBuffer>,
}

#[async_trait]
impl BufferedSource for WeatherBufferedSource {
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
        })?;
        Ok(())
    }
}
