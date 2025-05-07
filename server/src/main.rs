use chrono::{DateTime, Utc};
use config::ServerConfig;
use lifelog_core::uuid::Uuid;
use lifelog_server::server::proto::lifelog_server_service_server::LifelogServerServiceServer;
use lifelog_server::server::proto::FILE_DESCRIPTOR_SET;
use lifelog_server::server::Server as LifelogServer;
use tokio;
use tonic::transport::Server as TonicServer;
use tonic_reflection::server::Builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();
    let server = LifelogServer::new(&config).await?;

    let addr = format!("{}:{}", config.host, config.port).parse()?;

    println!("Starting server on {}", addr);
    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
        .build()?;

    let time: DateTime<Utc> = Utc::now();
    let uuid = Uuid::new_v4();

    let clonned_server = server.clone(); // TODO REMOVE THIS CLONE
    tokio::task::spawn(async move {
        clonned_server.policy_loop().await;
    });

    TonicServer::builder()
        .add_service(reflection_service)
        .add_service(LifelogServerServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
