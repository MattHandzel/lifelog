pub mod assertions;
pub mod device_client;
pub mod event_gen;
pub mod fault_layer;

use config::ServerConfig;
use fault_layer::{FaultController, FaultInjectionLayer};
use lifelog_server::grpc_service::GRPCServerLifelogServerService;
use lifelog_server::server::{Server, ServerHandle};
use lifelog_types::lifelog_server_service_client::LifelogServerServiceClient;
use lifelog_types::lifelog_server_service_server::LifelogServerServiceServer;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;
use tokio::time::{sleep, timeout};
use tonic::transport::Channel;
use utils::cas::FsCas;

use device_client::DeviceClient;

#[allow(dead_code)]
pub struct TestContext {
    pub server_addr: String,
    pub db_addr: String,
    pub db_process: Child,
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    pub client: LifelogServerServiceClient<Channel>,
    pub fault_controller: FaultController,
    pub cas_path: PathBuf,
    server_port: u16,
    db_port: u16,
}

impl TestContext {
    #[allow(dead_code)]
    pub async fn new() -> Self {
        Self::new_with_faults(FaultController::new()).await
    }

    pub async fn new_with_faults(fault_controller: FaultController) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_port = portpicker::pick_unused_port().expect("No ports available");
        let server_port = portpicker::pick_unused_port().expect("No ports available");

        let db_addr = format!("127.0.0.1:{}", db_port);
        let server_addr = format!("http://127.0.0.1:{}", server_port);
        let cas_path = temp_dir.path().join("cas");

        // Start SurrealDB
        let db_process = Command::new("surreal")
            .arg("start")
            .arg("--user")
            .arg("root")
            .arg("--pass")
            .arg("root")
            .arg("--bind")
            .arg(&db_addr)
            .arg("memory")
            .spawn()
            .expect("Failed to start SurrealDB");

        // Wait for DB to be ready
        sleep(Duration::from_secs(5)).await;

        // Integration tests run an ephemeral SurrealDB with root/root.
        // The server requires these env vars (see server/src/server.rs).
        std::env::set_var("LIFELOG_DB_USER", "root");
        std::env::set_var("LIFELOG_DB_PASS", "root");

        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: server_port as u32,
            database_endpoint: db_addr.clone(),
            database_name: "test_db".to_string(),
            server_name: "TestServer".to_string(),
            cas_path: cas_path.display().to_string(),
            default_correlation_window_ms: 30_000,
        };

        let server = timeout(Duration::from_secs(30), Server::new(&config))
            .await
            .expect("Timed out creating server (check DB connectivity / schema init)")
            .expect("Failed to create server");
        let server_handle = Arc::new(RwLock::new(server));
        let handle_clone = ServerHandle::new(server_handle.clone());
        let grpc_service = GRPCServerLifelogServerService {
            server: handle_clone.clone(),
        };

        // Start server background loop
        tokio::spawn(async move {
            handle_clone.r#loop().await;
        });

        let addr = format!("127.0.0.1:{}", server_port).parse().unwrap();
        let fault_layer = FaultInjectionLayer::new(fault_controller.clone());

        // Spawn the gRPC server with fault injection layer
        tokio::spawn(async move {
            tonic::transport::Server::builder()
                .layer(fault_layer)
                .add_service(LifelogServerServiceServer::new(grpc_service))
                .serve(addr)
                .await
                .expect("Server failed");
        });

        // Wait for server to be ready and fail fast if it never binds.
        sleep(Duration::from_millis(250)).await;

        let channel = timeout(
            Duration::from_secs(30),
            Channel::from_shared(server_addr.clone()).unwrap().connect(),
        )
        .await
        .expect("Timed out connecting to gRPC server (it may not have started)")
        .expect("Failed to connect to server");
        let client = LifelogServerServiceClient::new(channel);

        Self {
            server_addr,
            db_addr,
            db_process,
            temp_dir,
            client,
            fault_controller,
            cas_path,
            server_port,
            db_port,
        }
    }

    /// Get a clone of the primary gRPC client.
    pub fn client(&self) -> LifelogServerServiceClient<Channel> {
        self.client.clone()
    }

    /// Create N `DeviceClient` instances, each with a unique device_id.
    #[allow(dead_code)]
    pub fn create_device_clients(&self, n: usize) -> Vec<DeviceClient> {
        (0..n)
            .map(|i| DeviceClient::new(format!("device-{i}"), self.client()))
            .collect()
    }

    /// Get a `FsCas` handle pointing to the server's CAS directory.
    #[allow(dead_code)]
    pub fn cas(&self) -> FsCas {
        FsCas::new(&self.cas_path)
    }

    /// Get the server port (useful for reconnection scenarios).
    #[allow(dead_code)]
    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    /// Get the DB port (useful for direct DB assertions).
    #[allow(dead_code)]
    pub fn db_port(&self) -> u16 {
        self.db_port
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let _ = self.db_process.kill();
    }
}
