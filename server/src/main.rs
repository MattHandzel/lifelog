use async_trait::async_trait;
use config::CollectorConfig;
//use config::ServerConfig;
use chrono::{DateTime, Utc};
use config::ServerConfig;
use dashmap::DashMap;
use lifelog_core::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use surrealdb::engine::any;
use surrealdb::engine::local::Db;
use surrealdb::engine::local::Mem;
use surrealdb::engine::local::SurrealKV;
use surrealdb::Connect;
use surrealdb::Surreal;
use thiserror::Error;
use toml;
use tonic::{transport::Server as TonicServer, Response as TonicResponse, Status as TonicStatus};

use config::Config;
use lifelog_macros::lifelog_type;
use proto::lifelog_server_service_server::{LifelogServerService, LifelogServerServiceServer};
use proto::{
    GetConfigRequest, GetConfigResponse, GetDataRequest, GetDataResponse, GetStateRequest,
    GetStateResponse, RegisterCollectorRequest, RegisterCollectorResponse, ReportStateRequest,
    ReportStateResponse, SetConfigRequest, SetConfigResponse,
};

use uuid::Uuid;

use tokio::sync::RwLock;

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
#[derive(Clone, Debug, Serialize, Deserialize)]
struct RegisteredCollector {
    location: String,
    //connection
    sources: Vec<DataSource>,
    config: config::CollectorConfig,
}

#[derive(Clone)]
pub struct Server {
    db: Surreal<Db>,
    host: String,
    port: u16,
    collectors: Arc<RwLock<Vec<RegisteredCollector>>>,
}

impl Server {
    pub async fn new(config: &ServerConfig) -> Result<Self, ServerError> {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns("lifelog")
            .use_db(config.database_name.clone())
            .await?;

        Ok(Self {
            db,
            host: config.host.clone(),
            port: config.port,
            collectors: Arc::new(RwLock::new(Vec::new())),
        })
    }
}

//763 | /         async fn get_config(                                                                                                 ▐
//764 | |             &self,                                                                                                           ▐
//765 | |             request: tonic::Request<super::GetConfigRequest>,                                                                ▐
//766 | |         ) -> std::result::Result<                                                                                            ▐
//767 | |             tonic::Response<super::GetConfigResponse>,
//768 | |             tonic::Status,
//769 | |         >;
//    | |__________- `get_config` from trait
//770 | /         async fn set_config(
//771 | |             &self,
//772 | |             request: tonic::Request<super::SetConfigRequest>,
//773 | |         ) -> std::result::Result<
//774 | |             tonic::Response<super::SetConfigResponse>,
//775 | |             tonic::Status,
//776 | |         >;
//    | |__________- `set_config` from trait
//777 | /         async fn get_data(
//778 | |             &self,
//779 | |             request: tonic::Request<super::GetDataRequest>,
//780 | |         ) -> std::result::Result<tonic::Response<super::GetDataResponse>, tonic::Status>;
//    | |_________________________________________________________________________________________- `get_data` from trait
//781 | /         async fn report_state(
//782 | |             &self,
//783 | |             request: tonic::Request<super::ReportStateRequest>,
//784 | |         ) -> std::result::Result<
//785 | |             tonic::Response<super::ReportStateResponse>,
//786 | |             tonic::Status,
//787 | |         >;
//    | |__________- `report_state` from trait
#[tonic::async_trait]
impl LifelogServerService for Server {
    async fn register_collector(
        &self,
        request: tonic::Request<RegisterCollectorRequest>,
    ) -> Result<TonicResponse<RegisterCollectorResponse>, TonicStatus> {
        let inner = request.into_inner();

        //match file_type.as_str() {
        //    "toml" => {
        //
        //        let config = toml::from_str::<config::Config>(&inner.config).map_err(|e| {
        //            TonicStatus::invalid_argument(format!("Failed to parse config: {}", e))
        //        })?;
        //        // TODO: Parse the config and add it
        //        // TODO: Check to see if the collector is already added, if so throw an error
        //        let mut collectors = self.collectors.write().await;
        //        // TODO: Parse the input config to know about what data sources the collector has
        //
        //        (*collectors).push(RegisteredCollector {
        //            location: "todo".to_string(),
        //            sources: vec![],
        //            config: config,
        //        });
        //    }
        //    _ => {
        //        return Err(TonicStatus::invalid_argument(format!(
        //            "Unsupported file type {}",
        //            file_type
        //        )));
        //    }
        //}
        //
        //for collector in self.collectors.read().await.iter() {
        //    println!("Collector Config: {:?}", collector.config);
        //}

        Ok(TonicResponse::new(RegisterCollectorResponse {
            success: true,
            session_id: chrono::Utc::now().timestamp_subsec_nanos() as u64
                + chrono::Utc::now().timestamp() as u64,
        }))
    }

    async fn get_config(
        &self,
        _request: tonic::Request<GetConfigRequest>,
    ) -> Result<TonicResponse<GetConfigResponse>, TonicStatus> {
        println!("Received a get config request!");
        Ok(TonicResponse::new(GetConfigResponse::default()))
    }

    async fn set_config(
        &self,
        _request: tonic::Request<SetConfigRequest>,
    ) -> Result<TonicResponse<SetConfigResponse>, TonicStatus> {
        println!("Received a set config request!");
        Ok(TonicResponse::new(SetConfigResponse::default()))
    }

    async fn get_data(
        &self,
        _request: tonic::Request<GetDataRequest>,
    ) -> Result<TonicResponse<GetDataResponse>, TonicStatus> {
        println!("Received a get data request!");
        Ok(TonicResponse::new(GetDataResponse::default()))
    }

    async fn report_state(
        &self,
        _request: tonic::Request<ReportStateRequest>,
    ) -> Result<TonicResponse<ReportStateResponse>, TonicStatus> {
        let state = _request.into_inner().state.unwrap();

        println!(
            "Received a get state request! {} {:?}",
            state.name,
            state.timestamp.unwrap()
        );
        Ok(TonicResponse::new(ReportStateResponse {
            acknowledged: true,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig::default();
    let server = Server::new(&config).await?;

    let addr = format!("{}:{}", config.host, config.port).parse()?;

    println!("Starting server on {}", addr);
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let time: DateTime<Utc> = Utc::now();
    let uuid = Uuid::new_v4();
    TonicServer::builder()
        .add_service(service)
        .add_service(LifelogServerServiceServer::new(server))
        .serve(addr)
        .await?;

    Ok(())
}
