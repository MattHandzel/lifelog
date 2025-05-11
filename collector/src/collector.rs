use super::data_source::{DataSource, DataSourceError, DataSourceHandle};
use crate::modules::browser_history::BrowserHistorySource;
use crate::modules::screen::ScreenDataSource;
use config;
use data_modalities::{browser::BrowserFrame, screen::ScreenFrame};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::AbortHandle;
use tokio::time::Duration;

use config::{BrowserHistoryConfig, ScreenConfig};
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

struct RunningSource<C: Send + Sync + Debug + 'static> {
    instance: Arc<Mutex<Box<dyn DataSource<Config = C> + Send + Sync + 'static>>>,
    handle: DataSourceHandle,
}

impl<C: Send + Sync + Debug + 'static> fmt::Debug for RunningSource<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunningSource")
            .field("handle", &self.handle)
            // We can't directly debug the trait object, so we'll indicate its type
            .field("instance_type", &format!("{:?}", self.instance.type_id()))
            .finish()
    }
}

#[derive(Debug)]
pub enum CollectorError {
    SourceSetupError(String, Box<dyn std::error::Error + Send + Sync>),
    GrpcConnectionError(tonic::transport::Error),
    GrpcRequestError(tonic::Status),
    NotConnected,
    RegistrationFailed(String),
    Other(String),
}

impl std::fmt::Display for CollectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CollectorError::SourceSetupError(name, e) => {
                write!(f, "Source '{}' setup failed: {}", name, e)
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
            CollectorError::SourceSetupError(_, e) => Some(e.as_ref()),
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

pub struct Collector {
    task: Option<AbortHandle>,
    config: Arc<config::CollectorConfig>,
    sources: HashMap<String, Box<dyn RunningSourceTrait>>,

    grpc_client: Option<LifelogServerServiceClient<Channel>>,
    server_address: String,
    client_id: String,
}

trait RunningSourceTrait: Send + Sync + 'static + Debug + Any {}
impl<C: Send + Sync + 'static + Debug + Any> RunningSourceTrait for RunningSource<C> {}

/// The CollectorHandle is a struct that is used right now to abstract away how the collector
/// works. Right now, it is using a read-write lock but in the future I might want to change this
/// to the actor model.
#[derive(Clone)]
pub struct CollectorHandle {
    pub collector: Arc<RwLock<Collector>>,
}

impl CollectorHandle {
    pub fn new(collector: Collector) -> Self {
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
            {
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
            } // TODO: Need to drop the lock  here (maybe refactor it so that functions call for
              // lock)

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    pub async fn get_config(&self) -> Arc<config::CollectorConfig> {
        self.collector.read().await.config.clone()
    }

    pub async fn get_state(&self) -> CollectorState {
        self.collector.read().await._get_state().await
    }
}

// TODO: There needs to be some serious refactoring going on here or some better thinking. Right
// now I am just trying to gegt it to work but there the collector needs to be around a RWLock so
// the server can do stuff like editing it's config so these methods need to be refactored
// TODO: Implement pinging the server & re-trying upon disconnection
impl Collector {
    pub fn new(
        config: Arc<config::CollectorConfig>,
        server_address: String,
        client_id: String,
    ) -> Self {
        Self {
            task: None,
            config,
            sources: HashMap::new(),
            grpc_client: None,
            server_address,
            client_id,
        }
    }

    /// This function connects the collector to the main lifelog server, it initializes the
    /// grpc_client object and can be called at anytime to reconnect to the server.
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
        //self.handshake().await?; // TODO: Refactor, this, we shouldn't require a handshake in order
        //                         // to start logging, also, should we move control to the loop function>

        let config = Arc::clone(&self.config);

        self.sources.clear();

        let mut setup_errors: Vec<CollectorError> = Vec::new();

        if config.screen.enabled {
            let config_clone = Arc::clone(&self.config);
            match ScreenDataSource::new(config_clone.screen.clone()) {
                Ok(screen_source) => match screen_source.start() {
                    Ok(ds_handle) => {
                        let running_src = RunningSource::<ScreenConfig> {
                            instance: Arc::new(Mutex::new(Box::new(screen_source))),
                            handle: ds_handle,
                        };
                        self.sources
                            .insert("screen".to_string(), Box::new(running_src));
                    }
                    Err(e) => {
                        let err =
                            CollectorError::SourceSetupError("screen".to_string(), Box::new(e));
                        eprintln!("{}", err);
                        setup_errors.push(err);
                    }
                },
                Err(e) => {
                    let err = CollectorError::SourceSetupError("screen".to_string(), Box::new(e));
                    eprintln!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        if config.browser.enabled {
            let config_clone = Arc::clone(&self.config);
            match BrowserHistorySource::new(config_clone.browser.clone()) {
                Ok(browser_source) => match browser_source.start() {
                    Ok(ds_handle) => {
                        let running_src = RunningSource::<BrowserHistoryConfig> {
                            instance: Arc::new(Mutex::new(Box::new(browser_source))),
                            handle: ds_handle,
                        };
                        self.sources
                            .insert("browser".to_string(), Box::new(running_src));
                    }
                    Err(e) => {
                        let err =
                            CollectorError::SourceSetupError("browser".to_string(), Box::new(e));
                        eprintln!("{}", err);
                        setup_errors.push(err);
                    }
                },
                Err(e) => {
                    let err = CollectorError::SourceSetupError("browser".to_string(), Box::new(e));
                    eprintln!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        // For now I've removed the other sources.
        // But they should be added back here when they're reimplmemented!

        println!("Sources started. Active: {:?}", self.sources.keys());

        if let Err(e) = self.report_state().await {
            eprintln!("Failed to report initial status: {}", e);
            // Decide if this error should propagate or just be logged?
        }

        if !setup_errors.is_empty() {
            eprintln!("Warning: Some sources failed to start.");
        }

        Ok(())
    }

    async fn _get_state(&self) -> CollectorState {
        let mut source_states = Vec::<String>::new();
        let mut buffer_states = Vec::<String>::new();
        let mut total = 0;

        if let Some(running_src_trait) = self.sources.get("screen") {
            if let Some(running_screen_src) =
                (running_src_trait as &dyn Any).downcast_ref::<RunningSource<ScreenConfig>>()
            {
                let source_dyn_box_ref: &Arc<Mutex<
                    Box<dyn DataSource<Config = ScreenConfig> + Send + Sync + 'static>,
                >> = &running_screen_src.instance;
                let source_dyn_ref = &**source_dyn_box_ref;

                if let Some(screen_ds) =
                    (source_dyn_ref as &dyn Any).downcast_ref::<ScreenDataSource>()
                {
                    let buf_guard = screen_ds.buffer.lock().await;
                    let images = buf_guard.clone();

                    let screen_buf_size = images.capacity() * std::mem::size_of::<ScreenFrame>();

                    let fs = format!("Screen source buffer length: {}", screen_buf_size);
                    buffer_states.push(fs.to_string());

                    total += screen_buf_size; //TODO: get actual buffer size rather than just vec length

                    if let is_running = screen_ds.is_running() {
                        let fs = format!("Screen souce running state: {}", is_running);
                        source_states.push(fs.to_string());
                    } else {
                        source_states.push("Could not get screen source state".to_string());
                    }
                }
            }
        }

        CollectorState {
            name: self.client_id.clone(),
            timestamp: chrono::Utc::now(),
            source_states: source_states,
            source_buffer_sizes: buffer_states,
            total_buffer_size: total as u32,
        }
    }

    // Sends the current status to the gRPC server.
    pub async fn report_state(&mut self) -> Result<(), CollectorError> {
        let current_state: lifelog_proto::CollectorState = self._get_state().await.into();
        if let Some(client) = self.grpc_client.as_mut() {
            let active_sources: Vec<String> = self.sources.keys().cloned().collect();
            println!("Reporting status: Active sources = {:?}", active_sources);

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
        println!("Stopping Collector and sources...");

        if let Some(handle) = self.task.take() {
            handle.abort();
            println!("Aborted internal Collector task.");
        }

        for (name, _handle) in self.sources.drain() {
            println!("Stopping sources: {}", name);
            // handle.stop(); // Assuming SourceHandle has a stop method
        }
        self.sources.clear();

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
        println!("[gRPC] Starting data send...");
        const MAX_DATA_PER_CHANNEL: usize = 32;
        let (tx, rx) = tokio::sync::mpsc::channel(MAX_DATA_PER_CHANNEL);
        let collector_handle = self.collector.clone();

        tokio::spawn(async move {
            // TODO: Refactor this so that we never directly access the collector, the collector
            // handle should be the interface to the collector, this should just be a function we
            // call
            let images: Vec<ScreenFrame> = {
                let read_guard = collector_handle.collector.read().await;
                let running_source = (*read_guard.sources.get("screen").unwrap()).as_ref();
                let running: Option<&RunningSource<ScreenConfig>> =
                    (running_source as &dyn Any).downcast_ref::<RunningSource<ScreenConfig>>();

                println!("{:?}", running);

                if let Some(running_screen_src) = running {
                    let instance_arc = &running_screen_src.instance;
                    let mut guard = instance_arc.lock().await;
                    let boxed_dyn_data_source: &mut Box<dyn DataSource<Config = ScreenConfig> + Send + Sync + 'static> = &mut *guard;
                    let inner_dyn_data_source_ref: &mut (dyn DataSource<Config = ScreenConfig> + Send + Sync + 'static) = &mut **boxed_dyn_data_source;

                    if let Some(screen_ds_mut) = (inner_dyn_data_source_ref as &mut dyn Any).downcast_mut::<ScreenDataSource>() {
                        match screen_ds_mut.get_data().await {
                            Ok(images) => {
                                println!("[gRPC] clearing image buffer!");
                                screen_ds_mut.clear_buffer().await.unwrap_or_else(|e| eprintln!("Error clearing buffer: {}", e));
                                images
                            }
                            Err(e) => {
                                eprintln!("Failed to get buffer from ScreenDataSource! {:}", e);
                                Vec::new()
                            }
                        }
                    }
                    else {
                        eprintln!("[gRPC] could not downcast to ScreenDataSource");
                        Vec::new()
                    }

                } else {
                    eprintln!("[gRPC] 'screen' source not found or wrong type");
                    Vec::new()
                }
            };

            if images.is_empty() {
                println!("[gRPC] No images to send.");
            } else {
                println!("[gRPC] Sending {} images.", images.len());
            }

            for screen_frame in images {
                match <data_modalities::screen::ScreenFrame as TryInto<
                    lifelog_proto::ScreenFrame,
                >>::try_into(screen_frame)
                {
                    Ok(proto_frame) => {
                        let data_to_send = LifelogData {
                            payload: Some(lifelog_proto::lifelog_data::Payload::Screenframe(
                                proto_frame,
                            )),
                        };
                        if tx.send(Ok(data_to_send)).await.is_err() {
                            eprintln!("[gRPC] receiver dropped, stopping send");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("[gRPC] conversion error: {:?}", e);
                    }
                }
            }

            println!("[gRPC] Finished sending from ScreenDataSource.");

            let browser_entries: Vec<BrowserFrame> = {
                let read_guard = collector_handle.collector.read().await;
                let running_source = (*read_guard.sources.get("browser").unwrap()).as_ref();
                let running: Option<&RunningSource<BrowserHistoryConfig>> = (running_source
                    as &dyn Any)
                    .downcast_ref::<RunningSource<BrowserHistoryConfig>>();

                println!("{:?}", running);

                if let Some(running_browser_src) = running {
                    let instance_arc = &running_browser_src.instance;
                    let mut guard = instance_arc.lock().await;
                    let boxed_dyn_data_source: &mut Box<dyn DataSource<Config = BrowserHistoryConfig> + Send + Sync + 'static> = &mut *guard;
                    let inner_dyn_data_source_ref: &mut (dyn DataSource<Config = BrowserHistoryConfig> + Send + Sync + 'static) = &mut **boxed_dyn_data_source;

                    if let Some(browser_ds) = (inner_dyn_data_source_ref as &mut dyn Any).downcast_mut::<BrowserHistorySource>() {
                        match browser_ds.get_data() {
                            Ok(history) => history,
                            Err(e) => {
                                eprintln!("Failed to get buffer from BrowserHistorySource! {:}", e);
                                Vec::new()
                            }
                        }
                    } else {
                        eprintln!("[gRPC] could not downcast to BrowserHistorySource");
                        Vec::new()
                    }
                } else {
                    eprintln!("[gRPC] 'browser' source not found or wrong type");
                    Vec::new()
                }
            };

            if browser_entries.is_empty() {
                println!("[gRPC] No browser history to send.");
            } else {
                println!(
                    "[gRPC] Sending {} browser history entries.",
                    browser_entries.len()
                );
            }

            for browser_frame in browser_entries {
                match <data_modalities::browser::BrowserFrame as TryInto<
                    lifelog_proto::BrowserFrame,
                >>::try_into(browser_frame)
                {
                    Ok(proto_frame) => {
                        let data_to_send = LifelogData {
                            payload: Some(lifelog_proto::lifelog_data::Payload::Browserframe(
                                proto_frame,
                            )),
                        };
                        if tx.send(Ok(data_to_send)).await.is_err() {
                            eprintln!("[gRPC] receiver dropped, stopping send");
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("[gRPC] conversion error: {:?}", e);
                    }
                }
            }

            println!("[gRPC] Finished sending from BrowserHistorySource.");
        });

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}
