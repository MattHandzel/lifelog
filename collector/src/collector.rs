use super::logger::{DataLogger, LoggerHandle};
use crate::modules::{
    hyprland::HyprlandLogger, microphone::MicrophoneLogger, screen::ScreenLogger,
};
use config;
use data_modalities::screen::ScreenFrame;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::AbortHandle;
use tokio::time::Duration;

use futures_core::Stream;
use lifelog_types::CollectorState;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::transport::{Channel, Endpoint};
use tonic::Request;

use lifelog_proto::collector_service_server::CollectorService;
use lifelog_proto::lifelog_server_service_client::LifelogServerServiceClient;

use lifelog_proto::{
    GetCollectorConfigRequest, GetCollectorConfigResponse, GetCollectorStateResponse,
    GetDataRequest, GetDataResponse, GetStateRequest, LifelogData, RegisterCollectorRequest,
    ReportStateRequest, SetCollectorConfigRequest, SetCollectorConfigResponse,
};

use rand::distr::Distribution; // import the distribution trait o.w. our sampling doesn't work

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

pub struct GRPCServerCollectorService {
    pub collector: CollectorHandle,
}

pub struct Collector<T> {
    task: Option<AbortHandle>,
    config: Arc<config::CollectorConfig>,
    handles: HashMap<String, T>,

    grpc_client: Option<LifelogServerServiceClient<Channel>>,
    server_address: String,
    client_id: String,
}

/// The CollectorHandle is a struct that is used right now to abstract away how the collector
/// works. Right now, it is using a read-write lock but in the future I might want to change this
/// to the actor model.
#[derive(Clone)]
pub struct CollectorHandle {
    pub collector: Arc<RwLock<Collector<LoggerHandle>>>,
}

impl CollectorHandle {
    pub fn new(collector: Collector<LoggerHandle>) -> Self {
        Self {
            collector: Arc::new(RwLock::new(collector)),
        }
    }

    pub async fn start(&self) -> Result<(), CollectorError> {
        let collector = self.collector.clone();
        let mut collector = collector.write().await;
        collector.start().await
    }

    // NOTE: It is required to have this method grab the write lock every time so we don't have to
    // wait forever for the lock
    // TODO: Refacotr this function, it is just a dummy function right now
    pub async fn r#loop(&self) {
        let collector = self.collector.clone();
        loop {
            let mut collector = collector.write().await;
            if let Err(e) = collector.report_state().await {
                eprintln!("Failed to report state: {}", e);
                // Try and handshake again
                if let Err(e) = collector.handshake().await {
                    eprintln!("Failed to re-establish connection: {}", e);
                } else {
                    println!("Re-established connection.");
                }
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    pub async fn get_config(&self) -> Arc<config::CollectorConfig> {
        self.collector.read().await.config.clone()
    }

    pub async fn get_state(&self) -> CollectorState {
        self.collector.read().await._get_state()
    }
}

// TODO: There needs to be some serious refactoring going on here or some better thinking. Right
// now I am just trying to gegt it to work but there the collector needs to be around a RWLock so
// the server can do stuff like editing it's config so these methods need to be refactored
// TODO: Implement pinging the server & re-trying upon disconnection
impl Collector<LoggerHandle> {
    pub fn new(
        config: Arc<config::CollectorConfig>,
        server_address: String,
        client_id: String,
    ) -> Self {
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
        self.handshake().await?; // TODO: Refactor, this, we shouldn't require a handshake in order
                                 // to start logging, also, should we move control to the loop function>

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
        let current_state: lifelog_proto::CollectorState = self._get_state().into();
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

#[tonic::async_trait]
impl CollectorService for GRPCServerCollectorService {
    async fn get_state(
        &self,
        _request: tonic::Request<GetStateRequest>,
    ) -> Result<tonic::Response<GetCollectorStateResponse>, tonic::Status> {
        // we already have a helper that builds a CollectorState for us
        //let state: lifelog_proto::CollectorState = self._get_state().into();
        let state = self.collector.get_state().await.into();

        Ok(tonic::Response::new(GetCollectorStateResponse {
            state: Some(state),
        }))
    }

    async fn get_config(
        &self,
        _request: tonic::Request<GetCollectorConfigRequest>,
    ) -> Result<tonic::Response<GetCollectorConfigResponse>, tonic::Status> {
        //  CollectorConfig ↦ proto type conversion already used in `handshake`
        //  (so we can rely on the existing `Into` impl). :contentReference[oaicite:2]{index=2}:contentReference[oaicite:3]{index=3}

        let cfg_proto = self.collector.get_config().await.as_ref().clone().into();

        Ok(tonic::Response::new(GetCollectorConfigResponse {
            config: Some(cfg_proto),
        }))
    }

    async fn set_config(
        &self,
        request: tonic::Request<SetCollectorConfigRequest>,
    ) -> Result<tonic::Response<SetCollectorConfigResponse>, tonic::Status> {
        // TODO: Implement Full hot‑reload; for now we just acknowledge receipt.

        Ok(tonic::Response::new(SetCollectorConfigResponse {
            success: true,
        }))
    }

    // TODO: Refactor this so it's a stream?
    type GetDataStream = ReceiverStream<Result<LifelogData, tonic::Status>>;

    // NOTE: This utilizes a stream which, for large data sends (over 1MB) it is cheaper than doing
    // a unary RPC. Maybe in future don't stream all data.
    // TODO: Refactor this function so we can send data that is larger than 4MB (for example,
    // screenshots can easily get above 4MB)
    async fn get_data(
        &self,
        _request: tonic::Request<GetDataRequest>,
    ) -> Result<tonic::Response<Self::GetDataStream>, tonic::Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(8);

        let mut rng = rand::rng();
        // TODO: Replace this fake data with the real data buffer.
        let fake_data: Vec<ScreenFrame> = rand::distr::StandardUniform
            .sample_iter(&mut rng)
            .take(16)
            .collect(); // 16
                        // fake images
        tokio::spawn(async move {
            // TODO: For all messages we want to send, raise an error if the message is larger than
            // 4 MB. (TCP limit). We want to raise error here (how to fix that)
            for f in fake_data {
                let _ = tx
                    .send(Ok(lifelog_proto::LifelogData {
                        payload: Some(lifelog_proto::lifelog_data::Payload::Screenframe(f.into())), // TODO:
                                                                                                    // change name of screenframe so it matches the type
                    }))
                    .await
                    .unwrap();
            }
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}
