use sysinfo::{System, SystemExt};
use rusqlite::params;
use chrono::Utc;

pub async fn start_logger(
    config: &SystemPerformanceConfig,
    conn: &Connection,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut sys = System::new_all();
    
    loop {
        sys.refresh_all();
        
        let cpu_usage = sys.global_cpu_info().cpu_usage();
        let memory_used = sys.used_memory();
        let disk_used = sys.disks().iter().map(|d| d.total_space() - d.available_space()).sum();
        let network_up = sys.networks().iter().map(|(_, data)| data.transmitted()).sum();
        let network_down = sys.networks().iter().map(|(_, data)| data.received()).sum();

        conn.execute(
            "INSERT INTO system_metrics 
            (timestamp, cpu_usage, memory_used, disk_used, network_up, network_down)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                Utc::now().to_rfc3339(),
                cpu_usage,
                memory_used,
                disk_used,
                network_up,
                network_down
            ],
        )?;

        tokio::time::sleep(Duration::from_secs_f64(config.interval)).await;
    }
}
