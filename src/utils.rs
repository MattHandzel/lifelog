use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use std::path::{Path, PathBuf};

pub fn replace_home_dir_in_path(path: String) -> String {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    path.replace("~/", &format!("{}/", home_dir.to_str().unwrap()))
}

pub fn timestamp_to_epoch(timestamp: &str) -> Result<i64, &'static str> {
    if !timestamp.ends_with(".png") {
        return Err("Timestamp does not end with .png");
    }
    let timestamp = &timestamp[..timestamp.len() - 4]; // Remove the ".png" extension
    let format = "%Y-%m-%d_%H-%M-%S%.3f%:z";
    let datetime = DateTime::parse_from_str(timestamp, format).map_err(|_| "Failed to parse timestamp")?;
    Ok(datetime.timestamp())
}
