use serde::Deserialize;


#[derive(Deserialize)]
pub struct ScreenConfig {
    pub interval: u64,
    pub dir: String,
    pub program: String,
}

#[derive(Deserialize)]
pub struct Config {
    pub screen: ScreenConfig,
}


use std::fs;
use std::path::Path;
use toml;

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let content = fs::read_to_string(path).expect("Failed to read config file");
        toml::from_str(&content).expect("Failed to parse config file")
    }
}
