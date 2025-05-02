use super::logger::{DataLogger, LoggerHandle};
use crate::modules::{
    hyprland::HyprlandLogger, microphone::MicrophoneLogger, screen::ScreenLogger,
};
use config;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::{AbortHandle, JoinHandle};
use tokio::time::Duration;

use lifelog_core::CollectorState;
use tonic::transport::{Channel, Endpoint};
use tonic::Request;

pub mod proto {
    tonic::include_proto!("lifelog");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("lifelog_descriptor");
}

use proto::collector_service_server::{CollectorService, CollectorServiceServer};
use proto::lifelog_server_service_client::LifelogServerServiceClient;

use proto::{
    GetConfigRequest, GetConfigResponse, GetDataRequest, GetDataResponse, GetStateRequest,
    GetStateResponse, RegisterCollectorRequest, RegisterCollectorResponse, ReportStateRequest,
    ReportStateResponse, SetConfigRequest, SetConfigResponse,
};

use lifelog_core::DateTime;
use lifelog_core::Utc;
use lifelog_macros::lifelog_type;

use derive_more::{From, Into};

impl From<config::Config> for proto::Config {
    fn from(config: config::Config) -> Self {
        proto::Config {
            timestamp_format: config.timestamp_format,
            screen: Some(config.screen.into()),
            microphone: Some(config.microphone.into()),
            hyprland: Some(config.hyprland.into()),
            processes: Some(config.processes.into()),
            camera: Some(config.camera.into()),
        }
    }
}

impl From<config::ScreenConfig> for proto::ScreenConfig {
    fn from(config: config::ScreenConfig) -> Self {
        proto::ScreenConfig {
            enabled: config.enabled,
            interval: config.interval,
            output_dir: config.output_dir.to_str().unwrap_or("").to_string(),
            program: config.program,
            timestamp_format: config.timestamp_format,
        }
    }
}

impl From<config::MicrophoneConfig> for proto::MicrophoneConfig {
    fn from(config: config::MicrophoneConfig) -> Self {
        proto::MicrophoneConfig {
            enabled: config.enabled,
            output_dir: config.output_dir.to_str().unwrap_or("").to_string(),
            sample_rate: config.sample_rate,
            chunk_duration_secs: config.chunk_duration_secs,
            timestamp_format: config.timestamp_format,
            bits_per_sample: config.bits_per_sample,
            channels: config.channels,
            capture_interval_secs: config.capture_interval_secs,
        }
    }
}

impl From<config::HyprlandConfig> for proto::HyprlandConfig {
    fn from(config: config::HyprlandConfig) -> Self {
        proto::HyprlandConfig {
            enabled: config.enabled,
            interval: config.interval,
            output_dir: config.output_dir.to_str().unwrap_or("").to_string(),
            log_clients: config.log_clients,
            log_activewindow: config.log_activewindow,
            log_workspace: config.log_workspace,
            log_active_monitor: config.log_active_monitor,
            log_devices: config.log_devices,
        }
    }
}

impl From<config::CameraConfig> for proto::CameraConfig {
    fn from(config: config::CameraConfig) -> Self {
        proto::CameraConfig {
            enabled: config.enabled,
            interval: config.interval,
            output_dir: config.output_dir.to_str().unwrap_or("").to_string(),
            device: config.device,
            resolution: Some(config.resolution.into()),
            fps: config.fps,
            timestamp_format: config.timestamp_format,
        }
    }
}

impl From<config::Resolution> for proto::Resolution {
    fn from(resolution: config::Resolution) -> Self {
        proto::Resolution {
            width: resolution.width,
            height: resolution.height,
        }
    }
}

impl From<config::ProcessesConfig> for proto::ProcessesConfig {
    fn from(config: config::ProcessesConfig) -> Self {
        proto::ProcessesConfig {
            enabled: config.enabled,
            interval: config.interval,
            output_dir: config.output_dir.to_str().unwrap_or("").to_string(),
        }
    }
}

impl From<CollectorState> for proto::CollectorState {
    fn from(state: CollectorState) -> Self {
        proto::CollectorState {
            name: state.name,
            timestamp: Some(state.timestamp.to_proto()),
        }
    }
}

pub trait ProtoTimestamp {
    fn to_proto(self) -> prost_types::Timestamp;
}

impl ProtoTimestamp for chrono::DateTime<chrono::Utc> {
    fn to_proto(self) -> prost_types::Timestamp {
        prost_types::Timestamp {
            seconds: self.timestamp(),
            nanos: self.timestamp_subsec_nanos() as i32,
        }
    }
}

#[derive(Debug)]
pub enum CollectorError {
    LoggerSetupError(String, Box<dyn std::error::Error + Send + Sync>),
    GrpcConnectionError(tonic::transport::Error),
    GrpcRequestError(tonic::Status),
    NotConnected,
    RegistrationFailed(String),
    Other(String),
}

impl std::fmt::Display for CollectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectorError::LoggerSetupError(name, e) => {
                write!(f, "Logger '{}' setup failed: {}", name, e)
            }
            CollectorError::GrpcConnectionError(e) => write!(f, "gRPC connection failed: {}", e),
            CollectorError::GrpcRequestError(s) => write!(f, "gRPC request failed: {}", s),
            CollectorError::NotConnected => write!(f, "gRPC client not connected"),
            CollectorError::RegistrationFailed(msg) => {
                write!(f, "gRPC registration failed: {}", msg)
            }
            CollectorError::Other(msg) => write!(f, "Collector error: {}", msg),
        }
    }
}

impl std::error::Error for CollectorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CollectorError::LoggerSetupError(_, e) => Some(e.as_ref()),
            CollectorError::GrpcConnectionError(e) => Some(e),
            CollectorError::GrpcRequestError(s) => Some(s),
            _ => None,
        }
    }
}

impl From<tonic::transport::Error> for CollectorError {
    fn from(err: tonic::transport::Error) -> Self {
        CollectorError::GrpcConnectionError(err)
    }
}

impl From<tonic::Status> for CollectorError {
    fn from(status: tonic::Status) -> Self {
        CollectorError::GrpcRequestError(status)
    }
}

pub struct Collector<T> {
    task: Option<AbortHandle>,
    config: Arc<config::Config>,
    handles: HashMap<String, T>,

    grpc_client: Option<LifelogServerServiceClient<Channel>>,
    server_address: String,
    client_id: String,
}

impl Collector<LoggerHandle> {
    pub fn new(config: Arc<config::Config>, server_address: String, client_id: String) -> Self {
        Self {
            task: None,
            config,
            handles: HashMap::new(),
            grpc_client: None,
            server_address,
            client_id,
        }
    }

    pub async fn handshake(&mut self) -> Result<(), CollectorError> {
        println!("Attempting gRPC connection to {}...", self.server_address);

        let endpoint = Endpoint::from_shared(self.server_address.clone())
            .map_err(|e| CollectorError::Other(format!("Invalid server address: {}", e)))?
            .connect_timeout(Duration::from_secs(10));

        let channel = endpoint.connect().await?;
        let mut client = LifelogServerServiceClient::new(channel);

        println!("Connected. Performing handshake...");
        let request = Request::new(RegisterCollectorRequest {
            config: Some((*self.config).clone().into()),
        });

        let response = client.register_collector(request).await?;
        let handshake_response = response.into_inner();

        if handshake_response.success {
            println!(
                "Handshake successful. Session ID: {:?}",
                handshake_response.session_id
            );
            self.grpc_client = Some(client);
            Ok(())
        } else {
            let err_msg = format!("Server rejected handshake");
            eprintln!("{}", err_msg);
            Err(CollectorError::RegistrationFailed(err_msg))
        }
    }

    pub fn listen(&mut self) {}

    pub async fn start(&mut self) -> Result<(), CollectorError> {
        // need to handle error!
        self.handshake().await?;

        let config = Arc::clone(&self.config);

        self.handles.clear();

        let mut setup_errors: Vec<CollectorError> = Vec::new();

        if config.screen.enabled {
            let config_clone = Arc::clone(&self.config);
            match ScreenLogger::new(config_clone.screen.clone()) {
                Ok(logger) => match logger.setup() {
                    Ok(handle) => {
                        self.handles.insert("screen".to_string(), handle);
                    }
                    Err(e) => {
                        let err =
                            CollectorError::LoggerSetupError("screen".to_string(), Box::new(e));
                        eprintln!("{}", err);
                        setup_errors.push(err);
                    }
                },
                Err(e) => {
                    let err = CollectorError::LoggerSetupError("screen".to_string(), Box::new(e));
                    eprintln!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        if config.microphone.enabled {
            let config_clone = Arc::clone(&self.config);
            match MicrophoneLogger::new(config_clone.microphone.clone()) {
                Ok(logger) => match logger.setup() {
                    Ok(handle) => {
                        self.handles.insert("microphone".to_string(), handle);
                    }
                    Err(e) => {
                        let err =
                            CollectorError::LoggerSetupError("microphone".to_string(), Box::new(e));
                        eprintln!("{}", err);
                        setup_errors.push(err);
                    }
                },
                Err(e) => {
                    let err =
                        CollectorError::LoggerSetupError("microphone".to_string(), Box::new(e));
                    eprintln!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        if config.hyprland.enabled {
            let config_clone = Arc::clone(&self.config);
            match HyprlandLogger::new(config_clone.hyprland.clone()) {
                Ok(logger) => match logger.setup() {
                    Ok(handle) => {
                        self.handles.insert("hyprland".to_string(), handle);
                    }
                    Err(e) => {
                        let err =
                            CollectorError::LoggerSetupError("hyprland".to_string(), Box::new(e));
                        eprintln!("{}", err);
                        setup_errors.push(err);
                    }
                },
                Err(e) => {
                    let err = CollectorError::LoggerSetupError("hyprland".to_string(), Box::new(e));
                    eprintln!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        println!("Loggers started. Active: {:?}", self.handles.keys());

        if let Err(e) = self.report_state().await {
            eprintln!("Failed to report initial status: {}", e);
            // Decide if this error should propagate or just be logged?
        }

        if !setup_errors.is_empty() {
            eprintln!("Warning: Some loggers failed to start.");
        }

        Ok(())
    }

    fn _get_state(&self) -> CollectorState {
        CollectorState {
            name: self.client_id.clone(),
            timestamp: chrono::Utc::now(),
        }
    }

    // Sends the current status to the gRPC server.
    pub async fn report_state(&mut self) -> Result<(), CollectorError> {
        let current_state: proto::CollectorState = self._get_state().into();
        if let Some(client) = self.grpc_client.as_mut() {
            let active_loggers: Vec<String> = self.handles.keys().cloned().collect();
            println!("Reporting status: Active loggers = {:?}", active_loggers);

            let request = Request::new(ReportStateRequest {
                state: Some(current_state),
            });

            let response = client.report_state(request).await?;
            let status_response = response.into_inner();

            if status_response.acknowledged {
                println!("Server acknowledged status report.");
                Ok(())
            } else {
                let err_msg = "Server did not acknowledge status report".to_string();
                eprintln!("{}", err_msg);
                Err(CollectorError::Other(err_msg))
            }
        } else {
            eprintln!("Cannot report status: gRPC client not connected.");
            Err(CollectorError::NotConnected)
        }
    }

    pub fn send_data(&mut self) {}

    pub fn stop(&mut self) {
        println!("Stopping Collector and loggers...");

        if let Some(handle) = self.task.take() {
            handle.abort();
            println!("Aborted internal Collector task.");
        }

        for (name, handle) in self.handles.drain() {
            println!("Stopping logger: {}", name);
            // handle.stop(); // Assuming LoggerHandle has a stop method
        }
        self.handles.clear();

        self.grpc_client = None;
        println!("Collector stopped.");
    }

    pub async fn restart(&mut self) -> Result<(), CollectorError> {
        println!("Restarting Collector...");
        self.stop();
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.start().await
    }
}
