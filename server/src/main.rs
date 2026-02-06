use chrono::{DateTime, Utc};
use config::ServerConfig;
use lifelog_core::uuid::Uuid;
use lifelog_proto::lifelog_server_service_server::LifelogServerServiceServer;
use lifelog_server::server::GRPCServerLifelogServerService;
use lifelog_server::server::ServerHandle as LifelogServerHandle;

use lifelog_proto::FILE_DESCRIPTOR_SET;
use lifelog_server::server::Server as LifelogServer;
use tokio;
use tonic::transport::Server as TonicServer;
use tonic_reflection::server::Builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = ServerConfig::default();
    let server = LifelogServer::new(&config).await?;

    let addr = format!("{}:{}", config.host, config.port).parse()?;

    tracing::info!("Starting server on {}", addr);
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build_v1alpha()?; // This should be build_v1alpha otherwise the reflection gRPC service
                           // won't work with clients such as grpcui, it could be changed in the future

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_service_status("", tonic_health::ServingStatus::Serving)
        .await;

    let _time: DateTime<Utc> = Utc::now();
    let _uuid = Uuid::new_v4();

    let server_handle =
        LifelogServerHandle::new(std::sync::Arc::new(tokio::sync::RwLock::new(server)));
    let server_handle2 = server_handle.clone();

    tokio::task::spawn(async move {
        server_handle.r#loop().await;
    });

    TonicServer::builder()
        .add_service(reflection_service)
        .add_service(health_service)
        .add_service(LifelogServerServiceServer::new(
            GRPCServerLifelogServerService {
                server: server_handle2,
            },
        ))
        .serve(addr)
        .await?;

    Ok(())
}
