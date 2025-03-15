use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub screen_interval: u64,
    pub screen_dir: String,
    pub screen_program: String,
}
