use crate::setup::setup_weather_db;
use chrono::Utc;
use config::WeatherConfig;
use reqwest::Client;
use rusqlite::params;
use rusqlite::Connection;
use serde_json::Value;
use std::env;
use std::path::Path;
use std::time::Duration as StdDuration;
use tokio::time::sleep;
use tokio::time::Duration;

// Function to get API key from environment if available
fn get_weather_api_key(config_api_key: &str) -> String {
    env::var("WEATHER_API_KEY").unwrap_or_else(|_| config_api_key.to_string())
}

// TODO: How to get location based on IP that is resistant to vpn's
pub async fn start_logger(config: &WeatherConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting weather logger");
    let conn = setup_weather_db(Path::new(&config.output_dir)).unwrap();

    // Get API key from environment or config
    let api_key = get_weather_api_key(&config.api_key);

    if api_key.is_empty() {
        eprintln!("Weather API key is not set! Please set WEATHER_API_KEY environment variable or configure it in settings.");
        return Err("API key is missing".into());
    }

    let client = Client::new();
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=metric",
        config.latitude, config.longitude, api_key
    );

    loop {
        let response = client.get(&url).send().await;
        let json: Value = response.unwrap().json().await.unwrap();

        let main = json["main"].as_object().unwrap();
        let weather = json["weather"][0].as_object().unwrap();

        conn.execute(
            "INSERT INTO weather 
            (timestamp, temperature, humidity, pressure, conditions)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                Utc::now().to_rfc3339(),
                main["temp"].as_f64().unwrap(),
                main["humidity"].as_f64().unwrap(),
                main["pressure"].as_f64().unwrap(),
                weather["main"].as_str().unwrap()
            ],
        )?;

        sleep(StdDuration::from_secs_f64(config.interval)).await;
    }
}
