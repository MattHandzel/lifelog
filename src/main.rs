use tokio::join;
use lifelog::config::load_config;
use lifelog::modules::*;
use lifelog::setup;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Starting Life Logger!");
    let config = Arc::new(load_config());

    setup::initialize_project(&config).expect("Failed to initialize project");

    let mut tasks = Vec::new();
    if config.keyboard.enabled {
        let config = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            keyboard::start_logger(&config).await
        }));
    }
    if config.mouse.enabled {
        let config = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            mouse::start_logger(&config).await
        }));
    }
    if config.screen.enabled {
        let config = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            screen::start_logger(&config).await
        }));
    }

    if config.camera.enabled {
        let config = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            camera::start_logger(&config).await
        }));
    }
    if config.microphone.enabled {
        let config = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            microphone::start_logger(&config).await
        }));
    }

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
}
