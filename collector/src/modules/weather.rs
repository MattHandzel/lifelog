use crate::modules::polling_source::Capture;
use async_trait::async_trait;
use config::WeatherConfig;
use lifelog_core::{LifelogError, Utc, Uuid};
use lifelog_types::{to_pb_ts, WeatherFrame};
use prost::Message;
use reqwest::Client;
use serde_json::Value;
use std::env;
use tokio::time::Duration;

// Function to get API key from environment if available
fn get_weather_api_key(config_api_key: &str) -> String {
    env::var("WEATHER_API_KEY").unwrap_or_else(|_| config_api_key.to_string())
}

pub struct WeatherCapture {
    config: WeatherConfig,
    client: Client,
    url: String,
    api_key: String,
}

#[async_trait]
impl Capture for WeatherCapture {
    type Config = WeatherConfig;

    fn from_config(config: WeatherConfig) -> Result<Self, LifelogError> {
        let api_key = get_weather_api_key(&config.api_key);
        let client = Client::new();
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=metric",
            config.latitude, config.longitude, api_key
        );
        Ok(Self {
            config,
            client,
            url,
            api_key,
        })
    }

    fn output_dir(&self) -> &str {
        &self.config.output_dir
    }

    fn stream_id(&self) -> &str {
        "weather"
    }

    fn interval(&self) -> Duration {
        Duration::from_secs_f64(self.config.interval)
    }

    async fn init(&self) -> Result<(), LifelogError> {
        if self.api_key.is_empty() {
            tracing::error!("Weather API key is not set!");
            return Err(LifelogError::SourceSetup(
                "weather".to_string(),
                "API key is missing".to_string(),
            ));
        }
        Ok(())
    }

    async fn capture(&self) -> Result<Vec<Vec<u8>>, LifelogError> {
        let resp = self.client.get(&self.url).send().await.map_err(|e| {
            LifelogError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Weather API request failed: {e}"),
            ))
        })?;

        let json: Value = match resp.json::<Value>().await {
            Ok(j) => j,
            Err(_) => return Ok(vec![]),
        };

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
                    conditions: weather["main"].as_str().unwrap_or("Unknown").to_string(),
                    t_device: timestamp,
                    t_canonical: timestamp,
                    t_end: timestamp,
                    ..Default::default()
                };

                let mut buf = Vec::new();
                frame.encode(&mut buf).map_err(|e| {
                    LifelogError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("failed to encode WeatherFrame: {e}"),
                    ))
                })?;
                return Ok(vec![buf]);
            }
        }

        Ok(vec![])
    }
}
