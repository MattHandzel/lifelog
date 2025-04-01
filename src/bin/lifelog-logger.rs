use lifelog::config::*;
use lifelog::modules::{processes, screen};
use lifelog::setup;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() {
    #[cfg(feature = "dev")]
    println!("DEVELOPMENT MODE");

    println!("Starting Life Logger!");
    let config = load_config();
    let config = Arc::new(config);

    // Check to see if there is another instance of lifelog running
    if setup::is_already_running(env!("CARGO_PKG_NAME")) {
        println!("Another instance of lifelog is already running. Exiting...");

        #[cfg(not(feature = "dev"))]
        return;
    }

    setup::initialize_project(&config).expect("Failed to initialize project");

    let mut tasks = Vec::new();

    // Screen logger
    if config.screen.enabled {
        let config_clone = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            if let Err(e) = screen::start_logger(&config_clone.screen).await {
                eprintln!("Screen logger error: {}", e);
            }
        }));
    }

    // Process logger
    if config.processes.enabled {
        let config_clone = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            if let Err(e) = processes::start_logger(&config_clone.processes).await {
                eprintln!("Process logger error: {}", e);
            }
        }));
    }

    // Wait for all tasks to complete
    for task in tasks {
        if let Err(e) = task.await {
            eprintln!("Task join error: {}", e);
        }
    }
}
