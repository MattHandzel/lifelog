use chrono::{DateTime, Utc};
use config::TlsConfig;
use lifelog_core::uuid::Uuid;
use lifelog_server::grpc_service::GRPCServerLifelogServerService;
use lifelog_server::server::ServerHandle as LifelogServerHandle;
use lifelog_types::lifelog_server_service_server::LifelogServerServiceServer;

use lifelog_server::server::Server as LifelogServer;
use lifelog_types::FILE_DESCRIPTOR_SET;
use tonic::transport::Server as TonicServer;
use tonic_reflection::server::Builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let mut config = config::default_server_config();
    // Allow env var overrides for containerized deployments
    if let Ok(host) = std::env::var("LIFELOG_HOST") {
        config.host = host;
    }
    if let Ok(port) = std::env::var("LIFELOG_PORT") {
        if let Ok(p) = port.parse() {
            config.port = p;
        }
    }
    if let Ok(db) = std::env::var("LIFELOG_DB_ENDPOINT") {
        config.database_endpoint = db;
    }
    if let Ok(cas) = std::env::var("LIFELOG_CAS_PATH") {
        config.cas_path = cas;
    }
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

    let tls_config = TlsConfig::from_env();
    let mut builder = TonicServer::builder()
        .accept_http1(true)
        .layer(tonic_web::GrpcWebLayer::new());

    if let (Some(cert_path), Some(key_path)) = (&tls_config.cert_path, &tls_config.key_path) {
        let cert = std::fs::read_to_string(cert_path)?;
        let key = std::fs::read_to_string(key_path)?;
        let identity = tonic::transport::Identity::from_pem(cert, key);
        let tls = tonic::transport::ServerTlsConfig::new().identity(identity);
        builder = builder.tls_config(tls)?;
        tracing::info!("TLS enabled");
    }

    builder
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
