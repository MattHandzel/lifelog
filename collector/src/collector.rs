use super::data_source::{BufferedSource, DataSource, DataSourceHandle};
use crate::modules::browser_history::BrowserHistorySource;
use crate::modules::camera::CameraDataSource;
use crate::modules::processes::ProcessDataSource;
use crate::modules::screen::ScreenDataSource;
use async_trait::async_trait;
use config;
use mac_address::get_mac_address;
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::AbortHandle;
use tokio::time::Duration;

use config::{BrowserHistoryConfig, CameraConfig, ProcessesConfig, ScreenConfig};
use lifelog_core::*;
use lifelog_types::CollectorState;
use tokio::sync::mpsc;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream as ReceiverStreamWrapper;
use tokio_stream::StreamExt;
use tonic::transport::Endpoint;
use tonic::Request;

use lifelog_types::lifelog_server_service_client::LifelogServerServiceClient;
use lifelog_types::to_pb_ts;

use lifelog_types::{ControlMessage, RegisterCollectorRequest, ReportStateRequest};

pub mod upload_manager;

struct RunningSource<C: Send + Sync + Debug + 'static> {
    instance: Arc<Mutex<Box<dyn DataSource<Config = C> + Send + Sync + 'static>>>,
    handle: DataSourceHandle,
}

impl<C: Send + Sync + Debug + 'static> fmt::Debug for RunningSource<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunningSource")
            .field("handle", &self.handle)
            .field("instance_type", &format!("{:?}", self.instance.type_id()))
            .finish()
    }
}

pub struct Collector {
    task: Option<AbortHandle>,
    config: Arc<config::CollectorConfig>,
    sources: HashMap<String, Box<dyn RunningSourceTrait>>,

    control_tx: Option<mpsc::Sender<ControlMessage>>,
    upload_trigger: mpsc::Sender<()>,
    server_address: String,
    client_id: String,
}

#[async_trait]
impl<C: Send + Sync + Debug + Clone + 'static> RunningSourceTrait for RunningSource<C> {
    async fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>> {
        let guard = self.instance.lock().await;
        guard.get_buffered_source()
    }
}

#[async_trait]
trait RunningSourceTrait: Send + Sync + 'static + Debug {
    async fn get_buffered_source(&self) -> Option<Arc<dyn BufferedSource>>;
}

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

    pub async fn start(&self) -> Result<(), LifelogError> {
        let collector = self.collector.clone();
        let mut collector = collector.write().await;
        collector.start().await
    }

    pub async fn r#loop(&self) {
        let collector_handle = self.clone();
        loop {
            let needs_handshake = {
                let collector = collector_handle.collector.read().await;
                collector.control_tx.is_none()
            };

            if needs_handshake {
                let mut collector = collector_handle.collector.write().await;
                if let Err(e) = collector.handshake(collector_handle.clone()).await {
                    tracing::error!(error = %e, "Handshake failed, retrying in 5s");
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    continue;
                }
            }

            {
                let mut collector = collector_handle.collector.write().await;
                if let Err(e) = collector.report_state().await {
                    tracing::error!(error = %e, "Failed to report state, closing stream");
                    collector.control_tx = None;
                }
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }

    pub async fn get_config(&self) -> Arc<config::CollectorConfig> {
        self.collector.read().await.config.clone()
    }

    pub async fn get_state(&self) -> CollectorState {
        self.collector.read().await._get_state().await
    }
}

impl Collector {
    pub fn new(
        config: Arc<config::CollectorConfig>,
        server_address: String,
        client_id: String,
        upload_trigger: mpsc::Sender<()>,
    ) -> Self {
        Self {
            task: None,
            config,
            sources: HashMap::new(),
            control_tx: None,
            upload_trigger,
            server_address,
            client_id,
        }
    }

    pub async fn handshake(&mut self, handle: CollectorHandle) -> Result<(), LifelogError> {
        tracing::info!(addr = %self.server_address, "Attempting gRPC ControlStream connection");

        let mut endpoint = Endpoint::from_shared(self.server_address.clone())
            .map_err(|e| LifelogError::Database(format!("Invalid server address: {}", e)))?
            .connect_timeout(Duration::from_secs(10));

        if self.server_address.starts_with("https://") {
            let tls = tonic::transport::ClientTlsConfig::new().with_native_roots();
            endpoint = endpoint
                .tls_config(tls)
                .map_err(|e| LifelogError::Database(format!("TLS config error: {}", e)))?;
            tracing::info!("TLS enabled for server connection");
        }

        let channel = endpoint.connect().await?;
        let mut client = LifelogServerServiceClient::new(channel);

        let (tx, rx) = mpsc::channel::<ControlMessage>(128);
        self.control_tx = Some(tx.clone());

        let mac_addr = get_mac_address()
            .ok()
            .flatten()
            .map(|m| m.to_string().replace(":", ""))
            .unwrap_or_else(|| self.client_id.clone());
        let collector_id = mac_addr.clone();

        let reg_msg = ControlMessage {
            collector_id: collector_id.clone(),
            msg: Some(lifelog_types::control_message::Msg::Register(
                RegisterCollectorRequest {
                    config: Some((*self.config).clone()),
                },
            )),
        };
        tx.send(reg_msg)
            .await
            .map_err(|_| LifelogError::Database("Failed to send registration".to_string()))?;

        let stream_req = Request::new(ReceiverStreamWrapper::new(rx));
        let response = client.control_stream(stream_req).await?;
        let mut server_commands = response.into_inner();

        tokio::spawn(async move {
            tracing::info!("ControlStream established, listening for commands");
            while let Some(command_result) = server_commands.next().await {
                match command_result {
                    Ok(command) => {
                        tracing::info!(command = ?command.r#type, "Received server command");
                        if command.r#type == lifelog_types::CommandType::BeginUploadSession as i32 {
                            let coll = handle.collector.write().await;
                            let _ = coll.upload_trigger.try_send(());
                        }
                    }
                    Err(e) => {
                        tracing::error!("Server command stream error: {}", e);
                        break;
                    }
                }
            }
            tracing::warn!("ControlStream closed by server");
            let mut coll = handle.collector.write().await;
            coll.control_tx = None;
        });

        Ok(())
    }

    pub fn listen(&mut self) {}

    pub async fn start(&mut self) -> Result<(), LifelogError> {
        let config = Arc::clone(&self.config);
        self.sources.clear();
        let mut setup_errors: Vec<LifelogError> = Vec::new();

        if config.screen.as_ref().map(|s| s.enabled).unwrap_or(false) {
            let config_clone = Arc::clone(&self.config);
            match ScreenDataSource::new(config_clone.screen.clone().unwrap()) {
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
                        let err = LifelogError::SourceSetup("screen".to_string(), e.to_string());
                        tracing::error!("{}", err);
                        setup_errors.push(err);
                    }
                },
                Err(e) => {
                    let err = LifelogError::SourceSetup("screen".to_string(), e.to_string());
                    tracing::error!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        if config.browser.as_ref().map(|b| b.enabled).unwrap_or(false) {
            let config_clone = Arc::clone(&self.config);
            match BrowserHistorySource::new(config_clone.browser.clone().unwrap()) {
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
                        let err = LifelogError::SourceSetup("browser".to_string(), e.to_string());
                        tracing::error!("{}", err);
                        setup_errors.push(err);
                    }
                },
                Err(e) => {
                    let err = LifelogError::SourceSetup("browser".to_string(), e.to_string());
                    tracing::error!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        if config
            .processes
            .as_ref()
            .map(|p| p.enabled)
            .unwrap_or(false)
        {
            let config_clone = Arc::clone(&self.config);
            match ProcessDataSource::new(config_clone.processes.clone().unwrap()) {
                Ok(process_source) => match process_source.start() {
                    Ok(ds_handle) => {
                        let running_src = RunningSource::<ProcessesConfig> {
                            instance: Arc::new(Mutex::new(Box::new(process_source))),
                            handle: ds_handle,
                        };
                        self.sources
                            .insert("processes".to_string(), Box::new(running_src));
                    }
                    Err(e) => {
                        let err = LifelogError::SourceSetup("processes".to_string(), e.to_string());
                        tracing::error!("{}", err);
                        setup_errors.push(err);
                    }
                },
                Err(e) => {
                    let err = LifelogError::SourceSetup("processes".to_string(), e.to_string());
                    tracing::error!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        if config.camera.as_ref().map(|c| c.enabled).unwrap_or(false) {
            let config_clone = Arc::clone(&self.config);
            match CameraDataSource::new(config_clone.camera.clone().unwrap()) {
                Ok(camera_source) => match camera_source.start() {
                    Ok(ds_handle) => {
                        let running_src = RunningSource::<CameraConfig> {
                            instance: Arc::new(Mutex::new(Box::new(camera_source))),
                            handle: ds_handle,
                        };
                        self.sources
                            .insert("camera".to_string(), Box::new(running_src));
                    }
                    Err(e) => {
                        let err = LifelogError::SourceSetup("camera".to_string(), e.to_string());
                        tracing::error!("{}", err);
                        setup_errors.push(err);
                    }
                },
                Err(e) => {
                    let err = LifelogError::SourceSetup("camera".to_string(), e.to_string());
                    tracing::error!("{}", err);
                    setup_errors.push(err);
                }
            }
        }

        tracing::info!(active_sources = ?self.sources.keys(), "Sources started");

        if let Err(e) = self.report_state().await {
            tracing::error!(error = %e, "Failed to report initial status");
        }

        if !setup_errors.is_empty() {
            tracing::warn!("Some sources failed to start");
        }

        Ok(())
    }

    pub async fn get_buffered_sources(&self) -> Vec<Arc<dyn BufferedSource>> {
        let mut buffered = Vec::new();
        for source in self.sources.values() {
            if let Some(bs) = source.get_buffered_source().await {
                buffered.push(bs);
            }
        }
        buffered
    }

    pub fn get_upload_trigger_for_test(&self) -> mpsc::Sender<()> {
        self.upload_trigger.clone()
    }

    async fn _get_state(&self) -> CollectorState {
        let mut source_states = Vec::<String>::new();
        let mut buffer_states = Vec::<String>::new();
        let mut total = 0;

        let mac_address_variable: Option<String> = match get_mac_address() {
            Ok(Some(mac_addr)) => {
                tracing::debug!(mac_addr = %mac_addr, "Resolved MAC address");
                Some(mac_addr.to_string())
            }
            Ok(None) => None,
            Err(_e) => None,
        };

        if let Some(running_src_trait) = self.sources.get("screen") {
            if let Some(running_screen_src) =
                (running_src_trait as &dyn Any).downcast_ref::<RunningSource<ScreenConfig>>()
            {
                let guard = running_screen_src.instance.lock().await;
                if let Some(screen_ds) = guard.as_any().downcast_ref::<ScreenDataSource>() {
                    let screen_buf_size = match screen_ds.buffer.get_uncommitted_size().await {
                        Ok(s) => s as usize,
                        Err(e) => {
                            tracing::error!("Failed to get buffer size: {}", e);
                            0
                        }
                    };

                    let fs = format!("Screen source buffer length: {}", screen_buf_size);
                    buffer_states.push(fs.to_string());

                    total += screen_buf_size;

                    let is_running = screen_ds.is_running();
                    let fs = format!("Screen souce running state: {}", is_running);
                    source_states.push(fs.to_string());
                }
            }
        }

        if let Some(running_src_trait) = self.sources.get("processes") {
            if let Some(running_proc_src) =
                (running_src_trait as &dyn Any).downcast_ref::<RunningSource<ProcessesConfig>>()
            {
                let guard = running_proc_src.instance.lock().await;
                if let Some(proc_ds) = guard.as_any().downcast_ref::<ProcessDataSource>() {
                    let buf_size = match proc_ds.buffer.get_uncommitted_size().await {
                        Ok(s) => s as usize,
                        Err(e) => {
                            tracing::error!("Failed to get buffer size: {}", e);
                            0
                        }
                    };

                    let fs = format!("Processes source buffer length: {}", buf_size);
                    buffer_states.push(fs.to_string());
                    total += buf_size;

                    let is_running = proc_ds.is_running();
                    let fs = format!("Processes source running state: {}", is_running);
                    source_states.push(fs.to_string());
                }
            }
        }

        if let Some(running_src_trait) = self.sources.get("camera") {
            if let Some(running_cam_src) =
                (running_src_trait as &dyn Any).downcast_ref::<RunningSource<CameraConfig>>()
            {
                let guard = running_cam_src.instance.lock().await;
                if let Some(cam_ds) = guard.as_any().downcast_ref::<CameraDataSource>() {
                    let buf_size = match cam_ds.buffer.get_uncommitted_size().await {
                        Ok(s) => s as usize,
                        Err(e) => {
                            tracing::error!("Failed to get buffer size: {}", e);
                            0
                        }
                    };

                    let fs = format!("Camera source buffer length: {}", buf_size);
                    buffer_states.push(fs.to_string());
                    total += buf_size;

                    let is_running = cam_ds.is_running();
                    let fs = format!("Camera source running state: {}", is_running);
                    source_states.push(fs.to_string());
                }
            }
        }

        let dev_name = match mac_address_variable {
            Some(s) => s,
            None => self.client_id.clone(),
        };

        CollectorState {
            name: dev_name,
            timestamp: to_pb_ts(chrono::Utc::now()),
            source_states,
            source_buffer_sizes: buffer_states,
            total_buffer_size: total as u32,
            last_seen: None,
            upload_lag_bytes: 0,
            last_upload_time: None,
        }
    }

    pub async fn report_state(&mut self) -> Result<(), LifelogError> {
        let current_state = self._get_state().await;
        let mac_addr = get_mac_address()
            .ok()
            .flatten()
            .map(|m| m.to_string().replace(":", ""))
            .unwrap_or_else(|| self.client_id.clone());

        if let Some(tx) = self.control_tx.as_ref() {
            let active_sources: Vec<String> = self.sources.keys().cloned().collect();
            tracing::info!(active_sources = ?active_sources, "Reporting status via ControlStream");

            let msg = ControlMessage {
                collector_id: mac_addr,
                msg: Some(lifelog_types::control_message::Msg::State(
                    ReportStateRequest {
                        state: Some(current_state),
                    },
                )),
            };

            tx.send(msg)
                .await
                .map_err(|_| LifelogError::Database("Failed to send state report".to_string()))?;
            Ok(())
        } else {
            tracing::error!("Cannot report status: ControlStream not established");
            Err(LifelogError::NotConnected)
        }
    }

    pub fn stop(&mut self) {
        tracing::info!("Stopping Collector and sources");

        if let Some(handle) = self.task.take() {
            handle.abort();
            tracing::info!("Aborted internal Collector task");
        }

        for (name, _handle) in self.sources.drain() {
            tracing::info!(source = %name, "Stopping source");
        }
        self.sources.clear();

        self.control_tx = None;
        tracing::info!("Collector stopped");
    }

    pub async fn restart(&mut self) -> Result<(), LifelogError> {
        tracing::info!("Restarting Collector");
        self.stop();
        tokio::time::sleep(Duration::from_secs(1)).await;
        self.start().await
    }
}
