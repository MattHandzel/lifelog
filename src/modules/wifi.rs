use tokio::process::Command;
use rusqlite::params;

pub async fn start_logger(
    config: &WifiConfig,
    conn: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&config.scan_command)
            .output()
            .await?;

        let networks = parse_wifi_scan(&String::from_utf8(output.stdout)?);
        
        for network in networks {
            conn.execute(
                "INSERT INTO wifi_networks 
                (timestamp, ssid, bssid, signal_strength, frequency)
                VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    Utc::now().to_rfc3339(),
                    network.ssid,
                    network.bssid,
                    network.signal_strength,
                    network.frequency
                ],
            )?;
        }

        tokio::time::sleep(Duration::from_secs_f64(config.interval)).await;
    }
}
