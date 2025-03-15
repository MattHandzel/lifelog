use serde::Deserialize;
use std::path::PathBuf;
use std::fs;


#[derive(Debug, Deserialize)]
pub struct Config {
    pub keyboard: KeyboardConfig,
    pub mouse: MouseConfig,
    pub screen: ScreenConfig,
}

#[derive(Debug, Deserialize)]
pub struct KeyboardConfig {
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct MouseConfig {
    pub interval: f64,
    pub output_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct ScreenConfig {
    pub interval: f64,
    pub output_dir: String,
    pub program: String,
}

pub fn load_config() -> Config {

    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    let config_path: PathBuf = [home_dir.to_str().unwrap(), ".config/lifelog/config.toml"].iter().collect();
    let config_str = fs::read_to_string(config_path).expect("Failed to read config.toml");

    toml::from_str(&config_str).expect("Failed to parse config.toml")
}


