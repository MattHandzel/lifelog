use reqwest::Client;
use serde_json::Value;
use rusqlite::params;

pub async fn start_logger(
    config: &WeatherConfig,
    conn: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?lat={}&lon={}&appid={}&units=metric",
        config.latitude, config.longitude, config.api_key
    );

    loop {
        let response = client.get(&url).send().await?;
        let json: Value = response.json().await?;
        
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

        tokio::time::sleep(Duration::from_secs_f64(config.interval)).await;
    }
}
