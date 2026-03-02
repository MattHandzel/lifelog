use crate::ingest::SurrealIngestBackend;
use crate::server::ServerHandle;
use chrono::Utc;
use futures_core::Stream;
use lifelog_types::lifelog_server_service_server::LifelogServerService;
use lifelog_types::*;
use std::pin::Pin;
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};
use utils::ingest::ChunkIngester;

pub struct GRPCServerLifelogServerService {
    pub server: ServerHandle,
}

#[tonic::async_trait]
impl LifelogServerService for GRPCServerLifelogServerService {
    type ControlStreamStream = Pin<Box<dyn Stream<Item = Result<ServerCommand, Status>> + Send>>;

    async fn control_stream(
        &self,
        request: Request<Streaming<ControlMessage>>,
    ) -> Result<Response<Self::ControlStreamStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let server_handle = self.server.clone();

        tokio::spawn(async move {
            while let Some(msg_result) = stream.next().await {
                match msg_result {
                    Ok(msg) => {
                        let collector_id = msg.collector_id.clone();
                        match msg.msg {
                            Some(lifelog_types::control_message::Msg::Register(reg)) => {
                                tracing::info!(id = %collector_id, "Registering collector");
                                let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel(32);
                                let cmd_tx_clock = cmd_tx.clone();
                                let registered = crate::server::RegisteredCollector {
                                    id: collector_id.clone(),
                                    address: "unknown".to_string(), // TODO: extract from peer
                                    mac: collector_id.clone(),
                                    command_tx: cmd_tx,
                                    latest_config: reg.config,
                                };
                                server_handle.register_collector(registered).await;

                                // Forward commands from server to this collector stream
                                let tx_clone = tx.clone();
                                tokio::spawn(async move {
                                    while let Some(cmd) = cmd_rx.recv().await {
                                        let _ = tx_clone.send(cmd).await;
                                    }
                                });

                                // Periodic clock-sync pings to improve skew estimation.
                                // This exercises the command channel and allows collectors to report
                                // (device_now, backend_now) pairs.
                                tokio::spawn(async move {
                                    let mut interval =
                                        tokio::time::interval(Duration::from_secs(30));
                                    loop {
                                        interval.tick().await;
                                        let backend_now = Utc::now().to_rfc3339();
                                        let cmd = Ok(ServerCommand {
                                            r#type: CommandType::ClockSync as i32,
                                            payload: backend_now,
                                        });
                                        if cmd_tx_clock.send(cmd).await.is_err() {
                                            break;
                                        }
                                    }
                                });
                            }
                            Some(lifelog_types::control_message::Msg::State(report)) => {
                                if let Some(state) = report.state {
                                    server_handle.report_collector_state(state).await;
                                }
                            }
                            Some(lifelog_types::control_message::Msg::ClockSample(sample)) => {
                                let backend_now = sample.backend_now.and_then(|ts| {
                                    chrono::DateTime::from_timestamp(ts.seconds, ts.nanos as u32)
                                });

                                if let Some(ts) = sample.device_now {
                                    if let Some(device_now) = chrono::DateTime::from_timestamp(
                                        ts.seconds,
                                        ts.nanos as u32,
                                    ) {
                                        server_handle
                                            .handle_clock_sample(
                                                &collector_id,
                                                device_now,
                                                backend_now,
                                            )
                                            .await;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        tracing::error!("Control stream error: {}", e);
                        break;
                    }
                }
            }
            tracing::warn!("Collector disconnected from ControlStream");
        });

        Ok(Response::new(
            Box::pin(ReceiverStream::new(rx)) as Self::ControlStreamStream
        ))
    }

    async fn get_config(
        &self,
        _request: Request<GetSystemConfigRequest>,
    ) -> Result<Response<GetSystemConfigResponse>, Status> {
        let config = self.server.get_config().await;
        Ok(Response::new(GetSystemConfigResponse {
            config: Some(config),
        }))
    }

    async fn set_config(
        &self,
        request: Request<SetSystemConfigRequest>,
    ) -> Result<Response<SetSystemConfigResponse>, Status> {
        tracing::info!("Received set config request");
        let system_config = request
            .into_inner()
            .config
            .ok_or_else(|| Status::invalid_argument("missing system config payload"))?;

        self.server
            .apply_system_config(system_config)
            .await
            .map_err(|e| Status::internal(format!("failed to apply config: {e}")))?;

        Ok(Response::new(SetSystemConfigResponse { success: true }))
    }

    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        let query_message = request.into_inner().query.unwrap_or_default();
        tracing::info!(query = ?query_message, "Received query request");

        let keys = self
            .server
            .process_query(query_message)
            .await
            .map_err(|e| Status::internal(format!("Failed to process query: {}", e)))?;
        let proto_keys: Vec<lifelog_types::LifelogDataKey> = keys
            .iter()
            .map(|key| lifelog_types::LifelogDataKey {
                uuid: key.uuid.to_string(),
                origin: key.origin.get_table_name(),
            })
            .collect();
        let response = QueryResponse { keys: proto_keys };
        tracing::info!(count = response.keys.len(), "Query response");
        Ok(Response::new(response))
    }

    async fn replay(
        &self,
        request: Request<ReplayRequest>,
    ) -> Result<Response<ReplayResponse>, Status> {
        let req = request.into_inner();
        tracing::info!(req = ?req, "Received replay request");

        let resp = self
            .server
            .process_replay(req)
            .await
            .map_err(|e| Status::internal(format!("Failed to process replay: {}", e)))?;

        Ok(Response::new(resp))
    }

    async fn get_data(
        &self,
        request: Request<GetDataRequest>,
    ) -> Result<Response<GetDataResponse>, Status> {
        let inner_request = request.into_inner();
        tracing::info!(count = inner_request.keys.len(), "Received GetDataRequest");

        let keys = inner_request.keys;
        let data = self
            .server
            .get_data(keys)
            .await
            .map_err(|e| Status::internal(format!("Failed to get data: {}", e)))?;

        let response = GetDataResponse { data };
        tracing::info!(count = response.data.len(), "GetData response");
        Ok(Response::new(response))
    }

    async fn get_state(
        &self,
        _request: Request<GetStateRequest>,
    ) -> Result<Response<GetSystemStateResponse>, Status> {
        tracing::debug!("Received get state request");
        let state = self.server.get_state().await;
        Ok(Response::new(GetSystemStateResponse { state: Some(state) }))
    }

    async fn upload_chunks(
        &self,
        request: Request<Streaming<Chunk>>,
    ) -> Result<Response<Ack>, Status> {
        let mut stream = request.into_inner();
        let mut ingester: Option<ChunkIngester<SurrealIngestBackend>> = None;
        let mut last_stream: Option<lifelog_types::StreamIdentity> = None;
        let mut stream_id_str: Option<String> = None;
        let mut last_acked_offset = 0;

        while let Some(chunk_result) = stream.next().await {
            let chunk =
                chunk_result.map_err(|e| Status::internal(format!("Stream error: {}", e)))?;

            if ingester.is_none() {
                let stream_id = chunk
                    .stream
                    .as_ref()
                    .ok_or_else(|| Status::invalid_argument("missing stream identity"))?;
                last_stream = Some(stream_id.clone());
                stream_id_str = Some(stream_id.stream_id.clone());

                let server = self.server.server.read().await;
                let transforms = server.config.read().await.transforms.clone();
                let backend = SurrealIngestBackend {
                    db: server.db.clone(),
                    cas: server.cas.clone(),
                    skew_estimates: server.skew_estimates.clone(),
                    transforms,
                };
                ingester = Some(ChunkIngester::new(
                    backend,
                    server.cas.clone(),
                    stream_id.collector_id.clone(),
                    stream_id.stream_id.clone(),
                    stream_id.session_id,
                    0, // start_offset
                ));
            }

            if let Some(ref mut ing) = ingester {
                match ing
                    .apply_chunk(chunk.offset, &chunk.data, &chunk.hash)
                    .await
                {
                    Ok(_) => {
                        // Spec §6.2.1: ACK implies "fully queryable".
                        // If async work (e.g. derived OCR) is not complete, keep ACK pinned.
                        if ing.is_chunk_indexed(chunk.offset).await {
                            last_acked_offset = chunk.offset + chunk.data.len() as u64;
                        } else {
                            tracing::warn!(
                                stream_id = %stream_id_str.as_deref().unwrap_or_default(),
                                offset = chunk.offset,
                                "chunk persisted but not yet queryable; withholding ACK advance"
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("Ingest error: {}", e);
                        return Err(Status::internal(format!("Ingest error: {}", e)));
                    }
                }
            }
        }

        Ok(Response::new(Ack {
            stream: last_stream,
            acked_offset: last_acked_offset,
        }))
    }

    async fn get_upload_offset(
        &self,
        request: Request<GetUploadOffsetRequest>,
    ) -> Result<Response<GetUploadOffsetResponse>, Status> {
        let _inner = request.into_inner();
        let stream_id = _inner
            .stream
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("missing stream identity"))?;
        let db = self.server.server.read().await.db.clone();

        let mut response = db.query("SELECT * FROM upload_chunks WHERE collector_id = $c AND stream_id = $s AND session_id = $sess ORDER BY offset DESC LIMIT 1")
            .bind(("c", stream_id.collector_id.clone()))
            .bind(("s", stream_id.stream_id.clone()))
            .bind(("sess", stream_id.session_id))
            .await
            .map_err(|e| Status::internal(format!("Database error: {}", e)))?;

        #[derive(serde::Deserialize)]
        struct RawOffset {
            offset: u64,
            length: u64,
        }

        let results: Vec<RawOffset> = response
            .take(0)
            .map_err(|e| Status::internal(format!("Database take error: {}", e)))?;

        let offset = results.first().map(|r| r.offset + r.length).unwrap_or(0);

        Ok(Response::new(GetUploadOffsetResponse { offset }))
    }

    async fn list_modalities(
        &self,
        _request: Request<ListModalitiesRequest>,
    ) -> Result<Response<ListModalitiesResponse>, Status> {
        use lifelog_types::{FieldDescriptor, ModalityDescriptor};

        fn field(
            name: &str,
            ty: &str,
            display: &str,
            primary: bool,
            searchable: bool,
        ) -> FieldDescriptor {
            FieldDescriptor {
                name: name.to_string(),
                ty: ty.to_string(),
                display: display.to_string(),
                primary,
                searchable,
            }
        }

        fn modality(
            name: &str,
            stream_id: &str,
            category: &str,
            fields: Vec<FieldDescriptor>,
        ) -> ModalityDescriptor {
            ModalityDescriptor {
                name: name.to_string(),
                stream_id: stream_id.to_string(),
                category: category.to_string(),
                fields,
            }
        }

        let modalities = vec![
            modality(
                "Screen",
                "screen",
                "system",
                vec![
                    field("timestamp", "timestamp", "Captured At", true, false),
                    field("width", "i64", "Width", false, false),
                    field("height", "i64", "Height", false, false),
                    field("mime_type", "string", "Format", false, false),
                    field("blob_size", "i64", "Size", false, false),
                ],
            ),
            modality(
                "Browser History",
                "browser",
                "system",
                vec![
                    field("timestamp", "timestamp", "Visited At", true, false),
                    field("url", "string", "URL", true, true),
                    field("title", "string", "Title", true, true),
                    field("visit_count", "i64", "Visit Count", false, false),
                ],
            ),
            modality(
                "OCR Text",
                "ocr",
                "system",
                vec![
                    field("timestamp", "timestamp", "Extracted At", true, false),
                    field("text", "string", "Text", true, true),
                ],
            ),
            modality(
                "Audio",
                "audio",
                "system",
                vec![
                    field("timestamp", "timestamp", "Recorded At", true, false),
                    field("duration_secs", "f64", "Duration (s)", true, false),
                    field("codec", "string", "Codec", false, false),
                    field("sample_rate", "i64", "Sample Rate", false, false),
                    field("channels", "i64", "Channels", false, false),
                ],
            ),
            modality(
                "Microphone",
                "microphone",
                "system",
                vec![
                    field("timestamp", "timestamp", "Recorded At", true, false),
                    field("duration_secs", "f64", "Duration (s)", true, false),
                    field("codec", "string", "Codec", false, false),
                    field("sample_rate", "i64", "Sample Rate", false, false),
                    field("channels", "i64", "Channels", false, false),
                ],
            ),
            modality(
                "Keystrokes",
                "keystrokes",
                "system",
                vec![
                    field("timestamp", "timestamp", "Typed At", true, false),
                    field("text", "string", "Text", true, true),
                    field("application", "string", "Application", true, true),
                    field("window_title", "string", "Window", false, true),
                ],
            ),
            modality(
                "Clipboard",
                "clipboard",
                "system",
                vec![
                    field("timestamp", "timestamp", "Copied At", true, false),
                    field("text", "string", "Content", true, true),
                    field("mime_type", "string", "Format", false, false),
                ],
            ),
            modality(
                "Shell History",
                "shell_history",
                "system",
                vec![
                    field("timestamp", "timestamp", "Run At", true, false),
                    field("command", "string", "Command", true, true),
                    field("working_dir", "string", "Directory", false, true),
                    field("exit_code", "i64", "Exit Code", false, false),
                ],
            ),
            modality(
                "Window Activity",
                "window_activity",
                "system",
                vec![
                    field("timestamp", "timestamp", "Active At", true, false),
                    field("application", "string", "Application", true, true),
                    field("window_title", "string", "Window", true, true),
                    field("duration_secs", "f64", "Duration (s)", false, false),
                ],
            ),
            modality(
                "Mouse",
                "mouse",
                "system",
                vec![
                    field("timestamp", "timestamp", "Captured At", true, false),
                    field("x", "i64", "X", false, false),
                    field("y", "i64", "Y", false, false),
                    field("button", "string", "Button", false, false),
                    field("pressed", "bool", "Pressed", false, false),
                ],
            ),
            modality(
                "Processes",
                "processes",
                "system",
                vec![field("timestamp", "timestamp", "Sampled At", true, false)],
            ),
            modality(
                "Camera",
                "camera",
                "system",
                vec![
                    field("timestamp", "timestamp", "Captured At", true, false),
                    field("width", "i64", "Width", false, false),
                    field("height", "i64", "Height", false, false),
                    field("device", "string", "Device", false, false),
                    field("mime_type", "string", "Format", false, false),
                ],
            ),
            modality(
                "Weather",
                "weather",
                "system",
                vec![
                    field("timestamp", "timestamp", "Recorded At", true, false),
                    field("temperature", "f64", "Temperature (°C)", true, false),
                    field("humidity", "f64", "Humidity (%)", false, false),
                    field("pressure", "f64", "Pressure (hPa)", false, false),
                    field("conditions", "string", "Conditions", true, false),
                ],
            ),
            modality(
                "Hyprland",
                "hyprland",
                "system",
                vec![
                    field("timestamp", "timestamp", "Sampled At", true, false),
                    field("monitors", "array", "Monitors", false, false),
                    field("workspaces", "array", "Workspaces", false, false),
                    field("clients", "array", "Windows", false, false),
                ],
            ),
        ];

        Ok(Response::new(ListModalitiesResponse { modalities }))
    }

    async fn pair_collector(
        &self,
        request: Request<PairCollectorRequest>,
    ) -> Result<Response<PairCollectorResponse>, Status> {
        let client_hint = request
            .metadata()
            .get("x-lifelog-client-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().replace(':', ""))
            .filter(|s| !s.is_empty());
        let req = request.into_inner();
        let expected_token = std::env::var("LIFELOG_ENROLLMENT_TOKEN").unwrap_or_default();
        if expected_token.is_empty() {
            tracing::warn!("Server has no enrollment token configured");
            return Err(Status::internal("Enrollment not configured on server"));
        }

        if req.enrollment_token != expected_token {
            tracing::warn!("Invalid enrollment token attempt");
            return Err(Status::permission_denied("Invalid enrollment token"));
        }

        let collector_id =
            client_hint.unwrap_or_else(|| lifelog_core::uuid::Uuid::new_v4().to_string());
        tracing::info!(%collector_id, "Successfully paired collector");

        Ok(Response::new(PairCollectorResponse { collector_id }))
    }
}
