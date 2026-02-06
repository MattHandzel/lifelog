use clap::Parser;
use config::load_config;
use lifelog_collector::collector::Collector;
use lifelog_collector::collector::CollectorHandle;
use lifelog_collector::setup;
use std::sync::Arc;
use tokio;

#[derive(Parser, Debug)]
#[command(author, version, about = "LifeLog Logger Client", long_about = None)]
struct Cli {
    #[arg(
        short = 's',
        long = "server-address",
        value_name = "URL",
        default_value = "http://127.0.0.1:7182"
    )]
    server_address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // TODO: How to make it so that when the computer suspends all loggers are restarted so the
    // time is aligned
    #[cfg(feature = "dev")]
    tracing::info!("DEVELOPMENT MODE");

    let binary_name = std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .unwrap_or_else(|| "unknown".to_string());

    tracing::info!(binary = %binary_name, "Starting Life Logger");
    let config = Arc::new(load_config());

    // Check to see if there is another instance of lifelog running
    #[cfg(not(feature = "dev"))]
    if setup::n_processes_already_running(&binary_name, 2) {
        // need n=2 b/c when this process runs
        // it has same binary name as the one that is running
        tracing::warn!("Another instance of lifelog is already running. Exiting...");

        #[cfg(not(feature = "dev"))]
        return Ok(());
    }

    setup::initialize_project(&config).expect("Failed to initialize project");

    let cli = Cli::parse();

    let server_addr = cli.server_address;
    let client_id = config.id.clone();

    let collector = Arc::new(tokio::sync::RwLock::new(Collector::new(
        config,
        server_addr,
        client_id,
    )));
    let collector_handle = CollectorHandle {
        collector: collector.clone(),
    };

    tracing::info!("Starting Collector");
    let _ = collector_handle.start().await;
    collector_handle.r#loop().await;

    tracing::info!("Collector exiting");
    Ok(())
}
