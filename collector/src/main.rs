use config::load_config;
use lifelog_logger::logger::DataLogger;
use lifelog_logger::modules::*;
use lifelog_logger::setup;
use mobc::Pool;
use mobc_surrealdb::SurrealDBConnectionManager;
use std::env;
use std::process::Command;
use std::sync::Arc;
use std::{thread, time};

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

    // TODO: add windows support
    if !setup::n_processes_already_running("surreal", 1) {
        let home = env::var("HOME").expect("Unable to read home directory");
        let db_path = format!("rocksdb:{}/lifelog/data/db", home);

        #[cfg(target_os = "macos")]
        {
            Command::new("surreal")
                .arg("start")
                .arg("--user")
                .arg("root")
                .arg("--pass")
                .arg("root")
                .arg("--log")
                .arg("none")
                .arg("--no-banner")
                .arg(&db_path)
                .spawn()
                .expect("Failed to execute start surrealdb command");
        }
    }

    // let db spin up
    thread::sleep(time::Duration::from_millis(1000));

    if !setup::n_processes_already_running("surreal", 1) {
        panic!("Surreal db auto-launch failed. Please try again.");
    }

    setup::initialize_project(&config).expect("Failed to initialize project");

    let manager = SurrealDBConnectionManager::new(
        "127.0.0.1:8000", // localhost?
        "root",
        "root",
    );

    let pool = Pool::builder()
        .max_open(12) // number of loggers
        .max_idle(12) // don't force all to be active
        .max_lifetime(None) //don't kill
        .build(manager);

    let mut tasks = Vec::new();

    if config.screen.enabled {
        let config_clone = Arc::clone(&config);

        let conn = pool.get().await?;
        conn.use_ns("namespace").use_db("database").await?;

        tasks.push(tokio::spawn(async move {
            if let Err(e) = screen::start_logger(&config_clone.screen, &conn).await {
                eprintln!("Error in screen logger: {:?}", e);
            }
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

        let conn = pool.get().await?;
        conn.use_ns("namespace").use_db("database").await?;

        tasks.push(tokio::spawn(async move {
            if let Err(e) = processes::start_logger(&config_clone.processes, &conn).await {
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
