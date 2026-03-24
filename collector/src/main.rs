use clap::Parser;
use config::load_config;
use lifelog_collector::collector::Collector;
use lifelog_collector::collector::CollectorHandle;
use lifelog_collector::setup;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(author, version, about = "LifeLog Logger Client", long_about = None)]
struct Cli {
    #[arg(short = 's', long = "server-address", value_name = "URL")]
    server_address: Option<String>,
}

#[tokio::main]
#[allow(clippy::expect_used)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

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

    if std::env::var("WAYLAND_DISPLAY").is_err() && std::env::var("DISPLAY").is_err() {
        tracing::warn!("No display server detected (WAYLAND_DISPLAY and DISPLAY are both unset). Screen capture will not work.");
    }

    let config = Arc::new(load_config());

    let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/tmp"));
    let _pid_lock = match setup::PidLock::acquire(&runtime_dir) {
        Ok(lock) => lock,
        Err(msg) => {
            tracing::error!("{}", msg);
            return Ok(());
        }
    };

    setup::initialize_project(&config).expect("Failed to initialize project");

    let cli = Cli::parse();

    let server_addr = cli
        .server_address
        .unwrap_or_else(config::default_server_url);

    let client_id = config.id.clone();

    let (upload_mgr, upload_trigger) =
        lifelog_collector::collector::upload_manager::UploadManager::new(
            server_addr.clone(),
            config.id.clone(),
        );

    let collector = Arc::new(tokio::sync::RwLock::new(Collector::new(
        config.clone(),
        server_addr,
        client_id,
        upload_trigger,
    )));
    let collector_handle = CollectorHandle {
        collector: collector.clone(),
    };
    let collector_clone = collector.clone();
    tokio::spawn(async move {
        upload_mgr.run(collector_clone).await;
    });

    tracing::info!("Starting Collector");
    let _ = collector_handle.start().await;
    collector_handle.r#loop().await;

    tracing::info!("Collector exiting");
    Ok(())
}
