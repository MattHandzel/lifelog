use clap::Parser;
use config::load_config;
use lifelog_collector::collector::Collector;
use lifelog_collector::logger::DataLogger;
use lifelog_collector::modules::*;
use lifelog_collector::setup;
use lifelog_core::uuid::Uuid;
use lifelog_proto::collector_service_server::{CollectorService, CollectorServiceServer};
use lifelog_proto::FILE_DESCRIPTOR_SET;
use std::sync::Arc;
use std::thread;
use std::time;
use tokio;
use tonic::transport::Server as TonicServer;
use tonic_reflection::server::Builder;

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

    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()?;

    let cli = Cli::parse();

    let server_addr = cli.server_address;
    let id = Uuid::new_v4();
    let client_id = format!("client-{}", id);

    let addr = format!("{}:{}", config.host.clone(), config.port.clone()).parse()?;
    let collector = Arc::new(Collector::new(config, server_addr, client_id));
    //let cloned_collector = collector;

    // NOTE: CollectorServiceServer should be started before collector tries to connect to the
    // Server because the server is expecting to be able to connect to the collector
    let server_handle = tokio::spawn(async move {
        println!("Starting collector on {}", addr);
        tonic::transport::Server::builder()
            .add_service(reflection_service)
            .add_service(CollectorServiceServer::new(*collector.clone()))
            .serve(addr)
            .await
    });

    let collector_handle = tokio::spawn(async move { (*collector.clone()).start().await });

    use tokio::try_join;

    try_join!(server_handle)?; // or handle each individually

    println!("collector started successfully.");
    Ok(())
}
