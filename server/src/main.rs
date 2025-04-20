use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use surrealdb::engine::any;
use surrealdb::engine::local::Db;
use surrealdb::engine::local::Mem;
use surrealdb::engine::local::SurrealKV;
use surrealdb::Connect;
use surrealdb::Surreal;
use thiserror::Error;
use tonic::{transport::Server as TonicServer, Response as TonicResponse, Status as TonicStatus};

use proto::grpc_server_services_server::{GrpcServerServices, GrpcServerServicesServer};
use proto::{CollectorRegistrationRequest, CollectorRegistrationResponse};

pub mod proto {
    tonic::include_proto!("lifelog");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("lifelog_descriptor");
}

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] surrealdb::Error),
    #[error("Config error: {0}")]
    ConfigError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Tonic transport error: {0}")]
    TonicError(#[from] tonic::transport::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig<'a> {
    pub database_path: &'a str,
    pub host: String,
    pub port: u16,
    pub database_name: &'a str,
}

impl Default for ServerConfig<'_> {
    fn default() -> Self {
        Self {
            database_path: "surrealkv://",
            host: "127.0.0.1".to_string(),
            port: 8080,
            database_name: "main",
        }
    }
}

#[derive(Clone)]
pub struct LifeLogServer {
    db: Surreal<Db>,
    host: String,
    port: u16,
}

impl LifeLogServer {
    pub async fn new(config: &ServerConfig<'_>) -> Result<Self, ServerError> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("lifelog").use_db(config.database_name).await?;

        Ok(Self {
            db,
            host: config.host.clone(),
            port: config.port,
        })
    }
}

#[tonic::async_trait]
impl GrpcServerServices for LifeLogServer {
    async fn register_collector(
        &self,
        request: tonic::Request<CollectorRegistrationRequest>,
    ) -> Result<TonicResponse<CollectorRegistrationResponse>, TonicStatus> {
        let inner = request.into_inner();
        let config_string = inner.config;
        let file_type = inner.file_type;

        match file_type.as_str() {
            "toml" => {
                // TODO: Parse the config and add it
            }
            _ => {
                return Err(TonicStatus::invalid_argument(format!(
                    "Unsupported file type {}",
                    file_type
                )));
            }
        }

        Ok(TonicResponse::new(CollectorRegistrationResponse {
            success: true,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();
    let server = LifeLogServer::new(&config).await?;

    let addr = format!("{}:{}", config.host, config.port).parse()?;

    println!("Starting server on {}", addr);
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    TonicServer::builder()
        .add_service(service)
        .add_service(GrpcServerServicesServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
