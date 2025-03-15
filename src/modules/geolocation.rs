use reqwest::Client;
use rusqlite::params;

pub async fn start_logger(
    config: &GeoConfig,
    conn: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    loop {
        let location = if config.use_ip_fallback {
            get_ip_location(&client).await?
        } else {
            get_gps_location().await?
        };

        conn.execute(
            "INSERT INTO locations 
            (timestamp, latitude, longitude, accuracy, source)
            VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                Utc::now().to_rfc3339(),
                location.latitude,
                location.longitude,
                location.accuracy,
                location.source
            ],
        )?;

        tokio::time::sleep(Duration::from_secs_f64(config.interval)).await;
    }
}

async fn get_ip_location(client: &Client) -> Result<GeoData, Box<dyn std::error::Error>> {
    let response = client.get("http://ip-api.com/json/").send().await?;
    let json: Value = response.json().await?;
    
    Ok(GeoData {
        latitude: json["lat"].as_f64().unwrap(),
        longitude: json["lon"].as_f64().unwrap(),
        accuracy: 5000.0, // IP location accuracy is typically ~5km
        source: "IP".to_string(),
    })
}
