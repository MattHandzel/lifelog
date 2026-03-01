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

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about = "Lifelog Server Backend", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the gRPC server (default)
    Serve,
    /// Generate a secure random token for authentication
    GenerateToken,
}

fn check_auth(req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
    let auth_token = std::env::var("LIFELOG_AUTH_TOKEN")
        .map_err(|_| tonic::Status::internal("LIFELOG_AUTH_TOKEN must be configured"))?;
    let enrollment_token = std::env::var("LIFELOG_ENROLLMENT_TOKEN")
        .map_err(|_| tonic::Status::internal("LIFELOG_ENROLLMENT_TOKEN must be configured"))?;

    match req.metadata().get("authorization") {
        Some(t) => {
            let token_str = t.to_str().unwrap_or_default();
            if token_str == format!("Bearer {}", auth_token) {
                return Ok(req);
            }
            if token_str == format!("Bearer {}", enrollment_token) {
                return Ok(req);
            }
            tracing::warn!("Invalid authentication token provided");
            Err(tonic::Status::unauthenticated("Invalid token"))
        }
        None => {
            tracing::warn!("Unauthenticated connection attempt (no token provided)");
            Err(tonic::Status::unauthenticated("No token provided"))
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    if let Some(Commands::GenerateToken) = cli.command {
        let auth_token = Uuid::new_v4().to_string().replace("-", "");
        let enrollment_token = Uuid::new_v4().to_string().replace("-", "");
        println!("Generated LIFELOG_AUTH_TOKEN: {}", auth_token);
        println!("Generated LIFELOG_ENROLLMENT_TOKEN: {}", enrollment_token);
        return Ok(());
    }

    tracing_subscriber::fmt::init();

    let mut config = config::load_server_config_from_unified().unwrap_or_else(|| {
        panic!(
            "Missing or invalid [server] in {}. No defaults are applied.",
            std::env::var("LIFELOG_CONFIG_PATH")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|_| config::default_lifelog_config_path())
                .display()
        )
    });
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

    if !tls_config.is_enabled() {
        return Err(lifelog_core::LifelogError::Validation {
            field: "LIFELOG_TLS_CERT_PATH/LIFELOG_TLS_KEY_PATH".to_string(),
            reason: "must both be set; plaintext gRPC is not allowed".to_string(),
        }
        .into());
    }
    let cert_path = tls_config.cert_path.expect("checked above");
    let key_path = tls_config.key_path.expect("checked above");
    let cert = std::fs::read_to_string(&cert_path)?;
    let key = std::fs::read_to_string(&key_path)?;
    let identity = tonic::transport::Identity::from_pem(cert, key);
    let tls = tonic::transport::ServerTlsConfig::new().identity(identity);
    builder = builder.tls_config(tls)?;
    tracing::info!(cert = %cert_path, key = %key_path, "TLS enabled");

    let _auth_token = std::env::var("LIFELOG_AUTH_TOKEN").map_err(|_| {
        lifelog_core::LifelogError::Validation {
            field: "LIFELOG_AUTH_TOKEN".to_string(),
            reason: "must be set".to_string(),
        }
    })?;
    let _enrollment_token = std::env::var("LIFELOG_ENROLLMENT_TOKEN").map_err(|_| {
        lifelog_core::LifelogError::Validation {
            field: "LIFELOG_ENROLLMENT_TOKEN".to_string(),
            reason: "must be set".to_string(),
        }
    })?;

    builder
        .add_service(reflection_service)
        .add_service(health_service)
        .add_service(LifelogServerServiceServer::with_interceptor(
            GRPCServerLifelogServerService {
                server: server_handle2,
            },
            check_auth,
        ))
        .serve(addr)
        .await?;

    Ok(())
}
