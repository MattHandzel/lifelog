use tokio::join;
use lifelog::config::load_config;
use lifelog::modules::screen::*;

#[tokio::main]
async fn main() {
    println!("Starting Life Logger!");
    let config = load_config();

    join!(lifelog::modules::keyboard::start_keyboard_logger(&config),
          lifelog::modules::mouse::start_mouse_logger(&config),
          lifelog::modules::screen::capture_screenshots(&config));
}
