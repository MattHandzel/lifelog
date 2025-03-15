
use lifelog::config::Config;
use lifelog::modules::screen::*;

#[tokio::main]
async fn main() {
    let config = Config::from_file("config.toml");
    println!("Hello, world!");
    lifelog::modules::screen::capture_screenshots(&config).await;
}
