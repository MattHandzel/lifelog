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
        let config_clone = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            keyboard::start_logger(&config_clone.keyboard).await
        }));
    }
    if config.mouse.enabled {
        let config_clone = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            mouse::start_logger(&config_clone.mouse).await
        }));
    }
    if config.screen.enabled {
        let config_clone = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            screen::start_logger(&config_clone.screen).await
        }));
    }
    if config.camera.enabled {
        let config_clone = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            camera::start_logger(&config_clone.camera).await
        }));
    }

    if config.hyprland.enabled {
        let config_clone = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            hyprland::start_logger(&config_clone.hyprland).await
        }));
    }

    // Add to existing task spawning code
    if config.processes.enabled {
        let config_clone = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            processes::start_logger(&config_clone.processes).await
        }));
    }

    //if config.microphone.enabled {
    //    let config = Arc::clone(&config);
    //    tasks.push(tokio::spawn(async move {
    //        microphone::start_logger(&config).await
    //    }));
    //}

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
}
