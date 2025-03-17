use tokio::join;
use tokio::task;
use lifelog::config::load_config;
use lifelog::modules::*;
use lifelog::setup;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    #[cfg(feature = "dev")]
    println!("DEVELOPMENT MODE");

    println!("Starting Life Logger!");
    let config = Arc::new(load_config());

    // Check to see if there is another instance of lifelog running
    if setup::is_already_running(env!("CARGO_PKG_NAME")) {
        println!("Another instance of lifelog is already running. Exiting...");

        #[cfg(not(feature = "dev"))]
        return;
    }

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

    //if config.weather.enabled {
    //    let config_clone = Arc::clone(&config);
    //    tasks.push(tokio::spawn(async move {
    //        weather::start_logger(&config_clone.weather).await
    //    }));
    //}

    task::block_in_place(|| {
        let config_clone = Arc::clone(&config);
        tokio::runtime::Handle::current().block_on(async {
            microphone::start_logger(&config_clone.microphone).await;
        });
    });

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
}


