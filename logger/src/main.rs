use crate::modules::*;
use config::load_config;
use setup;
use std::env;
use std::sync::Arc;
use tokio::join;
use tokio::sync::RwLock;
use tokio::task;

#[tokio::main]
async fn main() {
    // TODO: How to make it so that when the computer suspends all loggers are restarted so the
    // time is aligned
    #[cfg(feature = "dev")]
    println!("DEVELOPMENT MODE");

    let binary_name = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "unknown".to_string());

    println!("Starting Life Logger! Binary: {}", binary_name);
    let config = Arc::new(load_config());

    // Check to see if there is another instance of lifelog running
    if setup::is_already_running(&binary_name) {
        println!("Another instance of lifelog is already running. Exiting...");

        #[cfg(not(feature = "dev"))]
        return;
    }

    setup::initialize_project(&config).expect("Failed to initialize project");

    let mut tasks = Vec::new();

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

    let user_is_running_wayland = env::var("WAYLAND_DISPLAY").is_ok();
    if config.input_logger.enabled {
        let config_clone = Arc::clone(&config);
        if user_is_running_wayland {
            tasks.push(tokio::spawn(async move {
                evdev_input_logger::start_logger(&config.input_logger).await;
            }));
        } else {
            tasks.push(tokio::spawn(async move {
                input_logger::start_logger(&config.input_logger).await;
            }));
        }
    }
    //if config.microphone.enabled {
    //    let config_clone = Arc::clone(&config);
    //    tasks.push(tokio::spawn(async move {
    //        microphone::start_logger(&config_clone.microphone).await;
    //    }));
    //}

    // Wait for all tasks to complete
    for task in tasks {
        let _ = task.await;
    }
}
