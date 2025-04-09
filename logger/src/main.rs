use config::load_config;
use lifelog_logger::logger::DataLogger;
use lifelog_logger::modules::*;
use lifelog_logger::setup;
use std::env;
use std::sync::Arc;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::{Object, Value};
use surrealdb::Surreal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    #[cfg(not(feature = "dev"))]
    if setup::n_processes_already_running(&binary_name, 2) {
        // need n=2 b/c when this process runs
        // it has same binary name as the one that is running
        println!("Another instance of lifelog is already running. Exiting...");

        #[cfg(not(feature = "dev"))]
        return Ok(());
    }

    if !setup::n_processes_already_running("surreal", 1) {
        panic!("Surreal db needs to be running. Please start it and try again.");
    }

    setup::initialize_project(&config).expect("Failed to initialize project");

    let mut db = Surreal::new::<Ws>("localhost:8000").await?;
    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    db.use_ns("namespace").use_db("database").await?;

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
        let logger = hyprland::HyprlandLogger::new(config.hyprland.clone()).unwrap();
        // 2. Initialize the logger
        if let Err(e) = logger.setup().await {
            eprintln!("Failed to initialize Hyprland logger: {}", e);
        }

        // 3. Spawn the logger task
        tasks.push(tokio::spawn(async move {
            match logger.run().await {
                Ok(_) => println!("Hyprland logger completed successfully"),
                Err(e) => eprintln!("Hyprland logger failed: {}", e),
            }
        }));
    }

    // Add to existing task spawning code
    if config.processes.enabled {
        let config_clone = Arc::clone(&config);
        tasks.push(tokio::spawn(async move {
            if let Err(e) = processes::start_logger(&config_clone.processes, &mut db).await {
                eprintln!("Error in processes logger: {:?}", e);
            }
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

    return Ok(());
}
