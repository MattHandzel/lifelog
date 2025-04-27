use tokio::task::{AbortHandle, JoinHandle};
use crate::modules::*;
use super::logger::{LoggerHandle, DataLogger};
use crate::modules::{
    screen::ScreenLogger,
    microphone::MicrophoneLogger,
    hyprland::HyprlandLogger,
};
use std::sync::Arc;
use tokio::time::Duration;
use config;
use std::collections::HashMap;

use tonic::transport::{Channel, Endpoint};
use tonic::Request;

pub mod grpc_client {
    tonic::include_proto!("controller");
}

use grpc_client::controller_service_client::ControllerServiceClient;
use grpc_client::{HandshakeRequest, ReportStatusRequest};

#[derive(Debug)]
pub enum ControllerError {
    LoggerSetupError(String, Box<dyn std::error::Error + Send + Sync>),
    GrpcConnectionError(tonic::transport::Error),
    GrpcRequestError(tonic::Status),
    NotConnected,
    HandshakeFailed(String),
    Other(String),
}

impl std::fmt::Display for ControllerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ControllerError::LoggerSetupError(name, e) => write!(f, "Logger '{}' setup failed: {}", name, e),
            ControllerError::GrpcConnectionError(e) => write!(f, "gRPC connection failed: {}", e),
            ControllerError::GrpcRequestError(s) => write!(f, "gRPC request failed: {}", s),
            ControllerError::NotConnected => write!(f, "gRPC client not connected"),
            ControllerError::HandshakeFailed(msg) => write!(f, "gRPC handshake failed: {}", msg),
            ControllerError::Other(msg) => write!(f, "Controller error: {}", msg),
        }
    }
}

impl std::error::Error for ControllerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
         match self {
            ControllerError::LoggerSetupError(_, e) => Some(e.as_ref()),
            ControllerError::GrpcConnectionError(e) => Some(e),
            ControllerError::GrpcRequestError(s) => Some(s),
            _ => None,
        }
    }
}

impl From<tonic::transport::Error> for ControllerError {
    fn from(err: tonic::transport::Error) -> Self {
        ControllerError::GrpcConnectionError(err)
    }
}

impl From<tonic::Status> for ControllerError {
    fn from(status: tonic::Status) -> Self {
        ControllerError::GrpcRequestError(status)
    }
}


pub struct Controller<T> {
    task: Option<AbortHandle>,
    config: Arc<config::Config>,
    handles: HashMap<String, T>,

    grpc_client: Option<ControllerServiceClient<Channel>>,
    server_address: String,
    client_id: String,
}

impl Controller<LoggerHandle> {
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

    pub async fn handshake(&mut self) -> Result<(), ControllerError> {
        println!("Attempting gRPC connection to {}...", self.server_address);

        let endpoint = Endpoint::from_shared(self.server_address.clone())
             .map_err(|e| ControllerError::Other(format!("Invalid server address: {}", e)))?
             .connect_timeout(Duration::from_secs(10));

        let channel = endpoint.connect().await?;
        let mut client = ControllerServiceClient::new(channel);

        println!("Connected. Performing handshake...");
        let request = Request::new(HandshakeRequest {
            client_id: self.client_id.clone(),
        });

        let response = client.handshake(request).await?;
        let handshake_response = response.into_inner();

        if handshake_response.success {
            println!("Handshake successful. Session ID: {:?}", handshake_response.session_id);
            self.grpc_client = Some(client);
            Ok(())
        } else {
            let err_msg = format!("Server rejected handshake: {}", handshake_response.message);
            eprintln!("{}", err_msg);
            Err(ControllerError::HandshakeFailed(err_msg))
        }
    }

    pub fn listen(&mut self) {

    }

    pub async fn start(&mut self) -> Result<(), ControllerError> {
        // need to handle error!
        self.handshake().await?;

        let config = Arc::clone(&self.config);

        self.handles.clear();

        let mut setup_errors: Vec<ControllerError> = Vec::new();

        if config.screen.enabled {
            let config_clone = Arc::clone(&self.config);
            match ScreenLogger::new(config_clone.screen.clone()) {
                Ok(logger) => {
                    match logger.setup() {
                         Ok(handle) => { self.handles.insert("screen".to_string(), handle); },
                         Err(e) => {
                            let err = ControllerError::LoggerSetupError("screen".to_string(), Box::new(e));
                            eprintln!("{}", err);
                            setup_errors.push(err);
                         }
                    }
                }
                Err(e) => {
                    let err = ControllerError::LoggerSetupError("screen".to_string(), Box::new(e));
                    eprintln!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        if config.microphone.enabled {
             let config_clone = Arc::clone(&self.config);
             match MicrophoneLogger::new(config_clone.microphone.clone()) {
                Ok(logger) => {
                    match logger.setup() {
                         Ok(handle) => { self.handles.insert("microphone".to_string(), handle); },
                         Err(e) => {
                            let err = ControllerError::LoggerSetupError("microphone".to_string(), Box::new(e));
                            eprintln!("{}", err);
                            setup_errors.push(err);
                         }
                    }
                }
                Err(e) => {
                    let err = ControllerError::LoggerSetupError("microphone".to_string(), Box::new(e));
                    eprintln!("{}", err);
                    setup_errors.push(err);
                }
             }
        }

        if config.hyprland.enabled {
            let config_clone = Arc::clone(&self.config);
            match HyprlandLogger::new(config_clone.hyprland.clone()) {
                Ok(logger) => {
                     match logger.setup() {
                         Ok(handle) => { self.handles.insert("hyprland".to_string(), handle); },
                         Err(e) => {
                            let err = ControllerError::LoggerSetupError("hyprland".to_string(), Box::new(e));
                            eprintln!("{}", err);
                            setup_errors.push(err);
                         }
                    }
                 }
                 Err(e) => {
                    let err = ControllerError::LoggerSetupError("hyprland".to_string(), Box::new(e));
                    eprintln!("{}", err);
                    setup_errors.push(err);
                 }
            }
        }

        println!("Loggers started. Active: {:?}", self.handles.keys());

        if let Err(e) = self.report_status().await {
            eprintln!("Failed to report initial status: {}", e);
            // Decide if this error should propagate or just be logged?
        }

        if !setup_errors.is_empty() {
             eprintln!("Warning: Some loggers failed to start.");
        }

        Ok(())
    }

    // Sends the current status to the gRPC server.
    pub async fn report_status(&mut self) -> Result<(), ControllerError> {
        if let Some(client) = self.grpc_client.as_mut() {
            let active_loggers: Vec<String> = self.handles.keys().cloned().collect();
            println!("Reporting status: Active loggers = {:?}", active_loggers);

            let request = Request::new(ReportStatusRequest {
                client_id: self.client_id.clone(),
                active_loggers,
            });

            let response = client.report_status(request).await?;
            let status_response = response.into_inner();

            if status_response.acknowledged {
                println!("Server acknowledged status report.");
                Ok(())
            } else {
                 let err_msg = "Server did not acknowledge status report".to_string();
                 eprintln!("{}", err_msg);
                 Err(ControllerError::Other(err_msg))
            }
        } else {
            eprintln!("Cannot report status: gRPC client not connected.");
            Err(ControllerError::NotConnected)
        }
    }

    pub fn send_data(&mut self) {

    }

    pub fn stop(&mut self) {
        println!("Stopping controller and loggers...");

        if let Some(handle) = self.task.take() {
            handle.abort();
            println!("Aborted internal controller task.");
        }

        for (name, handle) in self.handles.drain() {
             println!("Stopping logger: {}", name);
             // handle.stop(); // Assuming LoggerHandle has a stop method
        }
        self.handles.clear();

        // Optional: Notify the server that the client is shutting down
        // Needs another gRPC call, e.g., ReportStatus with empty list or a specific "goodbye" RPC
        // if let Some(client) = self.grpc_client.as_mut() {
        //     let client_id = self.client_id.clone(); // Clone before spawning task
        //     tokio::spawn(async move {
        //         let request = Request::new(ReportStatusRequest { client_id, active_loggers: vec![] });
        //         let _ = client.report_status(request).await; // Fire and forget or handle error
        //         println!("Sent final status report (empty).");
        //     });
        // }

        self.grpc_client = None;
        println!("Controller stopped.");
    }


    pub async fn restart(&mut self) -> Result<(), ControllerError> {
        println!("Restarting controller...");
        self.stop();
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.start().await
    }
}
