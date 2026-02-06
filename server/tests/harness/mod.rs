use std::process::{Child, Command};
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use tonic::transport::Channel;
use lifelog_proto::lifelog_server_service_client::LifelogServerServiceClient;
use config::ServerConfig;
use lifelog_server::server::{Server, GRPCServerLifelogServerService};
use lifelog_proto::lifelog_server_service_server::LifelogServerServiceServer;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TestContext {
    pub server_addr: String,
    pub db_addr: String,
    pub db_process: Child,
    #[allow(dead_code)]
    pub temp_dir: TempDir,
    pub client: LifelogServerServiceClient<Channel>,
}

impl TestContext {
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db_port = portpicker::pick_unused_port().expect("No ports available");
        let server_port = portpicker::pick_unused_port().expect("No ports available");
        
        let db_addr = format!("127.0.0.1:{}", db_port);
        let server_addr = format!("http://127.0.0.1:{}", server_port);

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

        // Wait for DB to be ready - increase to 1 second
        sleep(Duration::from_secs(1)).await;

        let config = ServerConfig {
            host: "127.0.0.1".to_string(),
            port: server_port as u32,
            database_endpoint: db_addr.clone(),
            database_name: "test_db".to_string(),
            server_name: "TestServer".to_string(),
            cas_path: temp_dir.path().join("cas").display().to_string(),
        };

        let server = Server::new(&config).await.expect("Failed to create server");
        let server_handle = Arc::new(RwLock::new(server));
        let grpc_service = GRPCServerLifelogServerService {
            server: lifelog_server::server::ServerHandle::new(server_handle),
        };

        let addr = format!("127.0.0.1:{}", server_port).parse().unwrap();
        
        // Spawn the gRPC server in a background task
        tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(LifelogServerServiceServer::new(grpc_service))
                .serve(addr)
                .await
                .expect("Server failed");
        });

        // Wait for server to be ready
        sleep(Duration::from_millis(200)).await;

        let channel = Channel::from_shared(server_addr.clone())
            .unwrap()
            .connect()
            .await
            .expect("Failed to connect to server");
        let client = LifelogServerServiceClient::new(channel);

        Self {
            server_addr,
            db_addr,
            db_process,
            temp_dir,
            client,
        }
    }

    pub fn client(&self) -> LifelogServerServiceClient<Channel> {
        self.client.clone()
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        let _ = self.db_process.kill();
    }
}
