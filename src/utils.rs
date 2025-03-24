use std::path::{Path, PathBuf};
use chrono::{DateTime, FixedOffset, NaiveDateTime,TimeZone}

pub fn replace_home_dir_in_path(path: String) -> String {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    path.replace("~/", &format!("{}/", home_dir.to_str().unwrap()))
}

pub fn timestamp_to_epoch(timestamp: &str) -> i64 {
    let timestamp = &timestamp[..timestamp.len() - 4]; // Remove the ".png" extension
    let format = "%Y-%m-%d_%H-%M-%S%.3f%:z";
    let datetime = DateTime::parse_from_str(timestamp, format).expect("Failed to parse timestamp");
    datetime.timestamp()
}
