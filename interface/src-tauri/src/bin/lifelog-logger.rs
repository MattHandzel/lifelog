use config::load_config;
use lifelog_interface_lib::setup;
use std::sync::Arc;

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
    let _config = Arc::new(load_config());

    // Check to see if there is another instance of lifelog running
    if setup::is_already_running(&binary_name) {
        println!("Another instance of lifelog is already running. Exiting...");

        #[cfg(not(feature = "dev"))]
        return;
    }

    setup::initialize_project().expect("Failed to initialize project");

    // The modules being referenced don't exist in the library
    // Instead we should be using the API client to communicate with the server
    println!("All modules will be handled by the server process.");
    println!("This binary is deprecated. Please use the lifelog-server-backend instead.");
}
