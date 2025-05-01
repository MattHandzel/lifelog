use clap::Parser;
use config::load_config;
use lifelog_collector::collector::Collector;
use lifelog_collector::logger::DataLogger;
use lifelog_collector::modules::*;
use lifelog_collector::setup;
use std::env;
use std::process::Command;
use std::sync::Arc;
use std::thread;
use std::time;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(author, version, about = "LifeLog Logger Client", long_about = None)]
struct Cli {
    #[arg(
        short = 's',
        long = "server-address",
        value_name = "URL",
        default_value = "http://localhost:50051"
    )]
    server_address: String,
}

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
    //if !setup::n_processes_already_running("surreal", 1) {
    //    let home = env::var("HOME").expect("Unable to read home directory");
    //    let db_path = format!("rocksdb:{}/lifelog/data/db", home);
    //
    //    #[cfg(target_os = "macos")]
    //    {
    //        Command::new("surreal")
    //            .arg("start")
    //            .arg("--user")
    //            .arg("root")
    //            .arg("--pass")
    //            .arg("root")
    //            .arg("--log")
    //            .arg("none")
    //            .arg("--no-banner")
    //            .arg(&db_path)
    //            .spawn()
    //            .expect("Failed to execute start surrealdb command");
    //    }
    //}

    // let db spin up
    //thread::sleep(time::Duration::from_millis(1000));
    //
    //if !setup::n_processes_already_running("surreal", 1) {
    //    panic!("Surreal db auto-launch failed. Please try again.");
    //}

    setup::initialize_project(&config).expect("Failed to initialize project");

    let cli = Cli::parse();

    let server_addr = cli.server_address;
    let id = Uuid::new_v4();
    let client_id = format!("client-{}", id);

    let mut collector = Collector::new(config, server_addr, client_id);
    collector.start().await?;

    println!("collector started successfully.");
    Ok(())
}
